use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::Query;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                sensor::resolve_sensors_by_str_ids,
                tree::{
                    NearestTreeListResponse, NearestTreeParams, ShareRequest, TransferRequest,
                    TreeCreateRequest, TreeListParams, TreeMarkerListResponse,
                    TreeMarkerQueryParams, TreeMarkerResponse, TreeResponse, TreeUpdateRequest,
                    TreeWithDistanceResponse,
                },
            },
            scope,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    sensor::SensorId,
    shared::{coordinates::Coordinate, distance::Distance, pagination::Pagination},
    tree::{PlantingYear, TreeMarker, TreeSearchQuery, TreeView},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_trees, create_tree))
        .routes(routes!(list_planting_years))
        .routes(routes!(list_tree_markers))
        .routes(routes!(get_nearest_trees))
        .routes(routes!(get_tree, update_tree, delete_tree))
        .routes(routes!(share_tree))
        .routes(routes!(revoke_share_tree))
        .routes(routes!(transfer_tree))
}

const TREE_LIST_Q_MAX_LEN: usize = 100;

#[utoipa::path(get, path = "/trees", tag = "Trees",
    operation_id = "listTrees",
    summary = "List all trees",
    description = "Returns a paginated list of all trees with their associated sensor data. \
                   Optional `q` parameter case-insensitively filters by tree number or species. \
                   Optional filter parameters (watering_status, has_cluster, planting_year) narrow the result; array parameters are repeatable.",
    params(TreeListParams),
    responses(
        (status = 200, description = "Paginated list of trees", body = ListResponse<TreeResponse>),
        (status = 400, description = "Invalid query parameter"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(query.len = tracing::field::Empty))]
pub async fn list_trees(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<TreeListParams>,
) -> Result<Json<ListResponse<TreeResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);

    let q = params
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    if let Some(ref qv) = q
        && qv.chars().count() > TREE_LIST_Q_MAX_LEN
    {
        return Err(ServiceError::InvalidInput(format!(
            "q must be at most {TREE_LIST_Q_MAX_LEN} characters"
        )));
    }

    if let Some(ref qv) = q {
        tracing::Span::current().record("query.len", qv.chars().count());
    }

    let watering_statuses = params
        .watering_status
        .into_iter()
        .map(domain::shared::watering_status::WateringStatus::from)
        .collect();

    let planting_years = params
        .planting_year
        .into_iter()
        .map(|y| PlantingYear::new(y as u32))
        .collect::<Result<Vec<_>, _>>()?;

    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Tree, Action::Read))
        .await?;

    let query = TreeSearchQuery {
        q,
        watering_statuses,
        has_cluster: params.has_cluster,
        planting_years,
        visible,
        ..TreeSearchQuery::default()
    };

    let page = state.tree_service.search_view(query, pagination).await?;

    let sensor_map = resolve_sensors_by_str_ids(
        &state.sensor_service,
        page.items.iter().filter_map(|t| t.sensor_id.as_deref()),
    )
    .await?;

    let response = ListResponse::from_page_with(page, &pagination, |tree: &TreeView| {
        let sensor = tree.sensor_id.as_deref().and_then(|id| sensor_map.get(id));
        TreeResponse::from((tree, sensor))
    });
    Ok(Json(response))
}

#[utoipa::path(get, path = "/trees/{tree_id}", tag = "Trees",
    operation_id = "getTree",
    summary = "Get a tree by ID",
    description = "Returns a single tree by its ID, including associated sensor data.",
    params(("tree_id" = uuid::Uuid, Path, description = "Tree ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn get_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let tree = state.tree_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Tree, Action::Read),
        &scope::effective_orgs(tree.organization_id, &tree.shared_with),
    )?;
    let sensor = match &tree.sensor_id {
        Some(sid) => {
            let sensor_id = SensorId::new(sid)?;
            Some(state.sensor_service.view_by_id(&sensor_id).await?)
        }
        None => None,
    };
    Ok(Json(TreeResponse::from((&tree, sensor.as_ref()))))
}

#[utoipa::path(post, path = "/trees", tag = "Trees",
    operation_id = "createTree",
    summary = "Create a new tree",
    description = "Creates a new tree with location and optional cluster or sensor association.",
    request_body = TreeCreateRequest,
    responses(
        (status = 201, description = "Tree created", body = TreeResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(entity): Json<TreeCreateRequest>,
) -> Result<(StatusCode, Json<TreeResponse>), ServiceError> {
    let org = scope::resolve_target_org(&state, user.id, entity.organization_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Tree, Action::Create),
            org,
        )
        .await?;
    let draft = entity.into_draft(org)?;
    let tree = state.tree_service.create(draft).await?;
    let view = state.tree_service.view_by_id(tree.id).await?;
    let sensor = match view.sensor_id.as_deref() {
        Some(sid) => {
            let sensor_id = SensorId::new(sid)?;
            Some(state.sensor_service.view_by_id(&sensor_id).await?)
        }
        None => None,
    };
    Ok((
        StatusCode::CREATED,
        Json(TreeResponse::from((&view, sensor.as_ref()))),
    ))
}

#[utoipa::path(put, path = "/trees/{tree_id}", tag = "Trees",
    operation_id = "updateTree",
    summary = "Update a tree",
    description = "Performs a full replacement update of a tree by its ID.",
    params(("tree_id" = uuid::Uuid, Path, description = "Tree ID")),
    request_body = TreeUpdateRequest,
    responses(
        (status = 200, description = "Tree updated", body = TreeResponse),
        (status = 403, description = "Missing tree:update in the owning organization or its shares"),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn update_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<TreeUpdateRequest>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let current = state.tree_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let effective_orgs = scope::effective_orgs(current.organization_id, &current.shared_with);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Tree, Action::Read),
        &effective_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Tree, Action::Update),
            &effective_orgs,
        )
        .await?;
    let create = TreeCreateRequest {
        species: entity.species,
        number: entity.number,
        planting_year: entity.planting_year,
        latitude: entity.latitude,
        longitude: entity.longitude,
        description: entity.description,
        tree_cluster_id: entity.tree_cluster_id,
        sensor_id: entity.sensor_id,
        provider: entity.provider,
        additional_information: entity.additional_information,
        organization_id: None,
    };
    let draft = create.into_draft(Id::new(current.organization_id))?;
    let tree = state.tree_service.replace(Id::new(id), draft).await?;
    let view = state.tree_service.view_by_id(tree.id).await?;
    let sensor = match view.sensor_id.as_deref() {
        Some(sid) => {
            let sensor_id = SensorId::new(sid)?;
            Some(state.sensor_service.view_by_id(&sensor_id).await?)
        }
        None => None,
    };
    Ok(Json(TreeResponse::from((&view, sensor.as_ref()))))
}

#[utoipa::path(delete, path = "/trees/{tree_id}", tag = "Trees",
    operation_id = "deleteTree",
    summary = "Delete a tree",
    description = "Permanently deletes a tree by its ID.",
    params(("tree_id" = uuid::Uuid, Path, description = "Tree ID")),
    responses(
        (status = 204, description = "Tree deleted"),
        (status = 403, description = "Missing tree:delete in the owning organization"),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn delete_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let current = state.tree_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Tree, Action::Read),
        &scope::effective_orgs(current.organization_id, &current.shared_with),
    )?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Tree, Action::Delete),
            Id::new(current.organization_id),
        )
        .await?;
    state.tree_service.delete(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/trees/planting-years", tag = "Trees",
    operation_id = "listPlantingYears",
    summary = "List distinct planting years",
    description = "Returns a list of distinct planting years across all trees.",
    responses(
        (status = 200, description = "List of distinct planting years", body = Vec<i32>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_planting_years(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<Vec<i32>>, ServiceError> {
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Tree, Action::Read))
        .await?;
    let years = state.tree_service.distinct_planting_years(visible).await?;
    Ok(Json(years.into_iter().map(|y| y.year() as i32).collect()))
}

#[utoipa::path(get, path = "/trees/nearest", tag = "Trees",
    operation_id = "getNearestTrees",
    summary = "Get nearest trees",
    description = "Finds trees closest to a given coordinate.",
    params(NearestTreeParams),
    responses(
        (status = 200, description = "Nearest trees", body = NearestTreeListResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_nearest_trees(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<NearestTreeParams>,
) -> Result<Json<NearestTreeListResponse>, ServiceError> {
    let limits = state.nearest_tree_limits;
    let coord = Coordinate::new(params.lat, params.lng)?;
    let radius = Distance::new(limits.max_radius_meters)?;
    let limit = params
        .limit
        .map(|l| l.min(u32::MAX as u64) as u32)
        .filter(|&l| l > 0)
        .unwrap_or(limits.default_limit)
        .min(limits.max_limit);

    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Tree, Action::Read))
        .await?;

    let results = state
        .tree_service
        .view_nearest(coord, radius, limit, visible)
        .await?;

    let sensor_map = resolve_sensors_by_str_ids(
        &state.sensor_service,
        results.iter().filter_map(|t| t.tree.sensor_id.as_deref()),
    )
    .await?;

    let data = results
        .iter()
        .map(|t| {
            let sensor = t
                .tree
                .sensor_id
                .as_deref()
                .and_then(|id| sensor_map.get(id));
            TreeWithDistanceResponse {
                tree: TreeResponse::from((&t.tree, sensor)),
                distance_meters: t.distance.meters(),
            }
        })
        .collect();

    Ok(Json(NearestTreeListResponse { data }))
}

#[utoipa::path(get, path = "/trees/markers", tag = "Trees",
    operation_id = "listTreeMarkers",
    summary = "List tree markers in a bounding box",
    description = "Returns minimal tree markers (id, lat, lng, watering_status, number, has_sensor) \
                   intersecting the given bounding box. Optional filter parameters narrow the result. \
                   Not paginated — the bounding box bounds the result.",
    params(TreeMarkerQueryParams),
    responses(
        (status = 200, description = "Marker list", body = TreeMarkerListResponse),
        (status = 400, description = "Invalid bbox or filter parameter"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_tree_markers(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<TreeMarkerQueryParams>,
) -> Result<Json<TreeMarkerListResponse>, ServiceError> {
    let bbox = params.parse_bbox().map_err(ServiceError::InvalidInput)?;

    let planting_years = params
        .planting_year
        .into_iter()
        .map(|y| PlantingYear::new(y as u32))
        .collect::<Result<Vec<_>, _>>()?;

    let watering_statuses = params
        .watering_status
        .into_iter()
        .map(domain::shared::watering_status::WateringStatus::from)
        .collect();

    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Tree, Action::Read))
        .await?;

    let query = TreeSearchQuery {
        watering_statuses,
        has_cluster: params.has_cluster,
        planting_years,
        bbox: Some(bbox),
        visible,
        ..Default::default()
    };

    let markers: Vec<TreeMarker> = state.tree_service.view_markers(query).await?;
    let data = markers.iter().map(TreeMarkerResponse::from).collect();
    Ok(Json(TreeMarkerListResponse { data }))
}

#[utoipa::path(post, path = "/trees/{tree_id}/shares", tag = "Trees",
    operation_id = "shareTree", summary = "Share a tree with a descendant organization",
    params(("tree_id" = uuid::Uuid, Path, description = "Tree ID")),
    request_body = ShareRequest,
    responses(
        (status = 204, description = "Share granted"),
        (status = 403, description = "Missing tree:update in owning organization"),
        (status = 404, description = "Tree or organization not found"),
        (status = 409, description = "Tree is part of a cluster"),
        (status = 422, description = "Target is not a proper descendant"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn share_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ShareRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = state.tree_service.view_by_id(Id::new(id)).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Tree, Action::Update),
            Id::new(current.organization_id),
        )
        .await?;
    state
        .tree_service
        .share_with(Id::new(id), Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/trees/{tree_id}/shares/{org_id}", tag = "Trees",
    operation_id = "revokeShareTree", summary = "Revoke a tree share",
    params(
        ("tree_id" = uuid::Uuid, Path, description = "Tree ID"),
        ("org_id" = uuid::Uuid, Path, description = "Organization ID to revoke the share from"),
    ),
    responses(
        (status = 204, description = "Share revoked"),
        (status = 403, description = "Missing tree:update in owning organization"),
        (status = 404, description = "Tree not found"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn revoke_share_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((id, org_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<StatusCode, ServiceError> {
    let current = state.tree_service.view_by_id(Id::new(id)).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Tree, Action::Update),
            Id::new(current.organization_id),
        )
        .await?;
    state
        .tree_service
        .revoke_share(Id::new(id), Id::new(org_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/trees/{tree_id}/organization", tag = "Trees",
    operation_id = "transferTree", summary = "Transfer a tree's ownership to another organization",
    description = "Moves a clusterless tree (and any attached sensor) to a different owning \
                   organization. Requires `tree:update` in both the source and target organization.",
    params(("tree_id" = uuid::Uuid, Path, description = "Tree ID")),
    request_body = TransferRequest,
    responses(
        (status = 204, description = "Tree transferred"),
        (status = 403, description = "Missing tree:update in source or target organization"),
        (status = 404, description = "Tree or organization not found"),
        (status = 409, description = "Tree is part of a cluster"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = %id))]
pub async fn transfer_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = state.tree_service.view_by_id(Id::new(id)).await?;
    let perm = Permission::new(Resource::Tree, Action::Update);
    state
        .authorization_service
        .require(user.id, perm, Id::new(current.organization_id))
        .await?;
    state
        .authorization_service
        .require(user.id, perm, Id::new(req.organization_id))
        .await?;
    state
        .tree_service
        .transfer(Id::new(id), Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
