use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                tree::{
                    NearestTreeListResponse, NearestTreeParams, TreeCreateRequest,
                    TreeMarkerListResponse, TreeMarkerQueryParams, TreeMarkerResponse,
                    TreeResponse, TreeUpdateRequest,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    sensor::SensorId,
    shared::{error::ValidationError, pagination::Pagination},
    tree::{PlantingYear, TreeDraft, TreeMarker, TreeSearchQuery, TreeView},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_trees, create_tree))
        .routes(routes!(list_planting_years))
        .routes(routes!(list_tree_markers))
        .routes(routes!(get_nearest_trees))
        .routes(routes!(get_tree, update_tree, delete_tree))
}

#[utoipa::path(get, path = "/trees", tag = "Trees",
    operation_id = "listTrees",
    summary = "List all trees",
    description = "Returns a paginated list of all trees with their associated sensor data.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of trees", body = ListResponse<TreeResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_trees(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .tree_service
        .search_view(TreeSearchQuery::default(), pagination)
        .await?;

    let sensor_ids: Vec<SensorId> = page
        .items
        .iter()
        .filter_map(|t| {
            t.sensor_id
                .as_deref()
                .map(SensorId::new)
                .and_then(Result::ok)
        })
        .collect();
    let sensors = state.sensor_service.view_by_ids(&sensor_ids).await?;
    let sensor_map: HashMap<&str, _> = sensors.iter().map(|s| (s.id.as_str(), s)).collect();

    let response = ListResponse::from_page_with(page, &pagination, |tree: &TreeView| {
        let sensor = tree
            .sensor_id
            .as_deref()
            .and_then(|id| sensor_map.get(id).copied());
        TreeResponse::from((tree, sensor))
    });
    Ok(Json(response))
}

#[utoipa::path(get, path = "/trees/{tree_id}", tag = "Trees",
    operation_id = "getTree",
    summary = "Get a tree by ID",
    description = "Returns a single tree by its ID, including associated sensor data.",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
pub async fn get_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let tree = state.tree_service.view_by_id(Id::from(id)).await?;
    let sensor = match &tree.sensor_id {
        Some(sid) => {
            let sensor_id =
                SensorId::new(sid).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
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
    Json(entity): Json<TreeCreateRequest>,
) -> Result<(StatusCode, Json<TreeResponse>), ServiceError> {
    let draft: TreeDraft = entity
        .try_into()
        .map_err(|e: ValidationError| ServiceError::InvalidInput(e.to_string()))?;
    let tree = state.tree_service.create(draft).await?;
    let view = state.tree_service.view_by_id(tree.id).await?;
    let sensor = match view.sensor_id.as_deref() {
        Some(sid) => {
            let sensor_id =
                SensorId::new(sid).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
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
    params(("tree_id" = i32, Path, description = "Tree ID")),
    request_body = TreeUpdateRequest,
    responses(
        (status = 200, description = "Tree updated", body = TreeResponse),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
pub async fn update_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<TreeUpdateRequest>,
) -> Result<Json<TreeResponse>, ServiceError> {
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
    };
    let draft: TreeDraft = create
        .try_into()
        .map_err(|e: ValidationError| ServiceError::InvalidInput(e.to_string()))?;
    let tree = state.tree_service.replace(Id::from(id), draft).await?;
    let view = state.tree_service.view_by_id(tree.id).await?;
    let sensor = match view.sensor_id.as_deref() {
        Some(sid) => {
            let sensor_id =
                SensorId::new(sid).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
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
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 204, description = "Tree deleted"),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
pub async fn delete_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.tree_service.delete(Id::from(id)).await?;
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
) -> Result<Json<Vec<i32>>, ServiceError> {
    let years = state.tree_service.distinct_planting_years().await?;
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
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<NearestTreeParams>,
) -> Result<Json<NearestTreeListResponse>, ServiceError> {
    todo!()
}

fn dto_status_to_domain(
    s: crate::http::v1::dto::WateringStatus,
) -> domain::shared::watering_status::WateringStatus {
    use crate::http::v1::dto::WateringStatus as Dto;
    use domain::shared::watering_status::WateringStatus as Dom;
    match s {
        Dto::Good => Dom::Good,
        Dto::Moderate => Dom::Moderate,
        Dto::Bad => Dom::Bad,
        Dto::JustWatered => Dom::JustWatered,
        Dto::Unknown => Dom::Unknown,
    }
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
    Query(params): Query<TreeMarkerQueryParams>,
) -> Result<Json<TreeMarkerListResponse>, ServiceError> {
    let bbox = params.parse_bbox().map_err(ServiceError::InvalidInput)?;

    let planting_years = params
        .planting_year
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|y| PlantingYear::new(y as u32))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;

    let watering_statuses = params
        .watering_status
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(dto_status_to_domain)
        .collect();

    let query = TreeSearchQuery {
        watering_statuses,
        has_cluster: params.has_cluster,
        planting_years,
        bbox: Some(bbox),
        ..Default::default()
    };

    let markers: Vec<TreeMarker> = state.tree_service.view_markers(query).await?;
    let data = markers.iter().map(TreeMarkerResponse::from).collect();
    Ok(Json(TreeMarkerListResponse { data }))
}
