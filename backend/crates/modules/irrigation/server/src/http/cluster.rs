use std::{collections::HashMap, sync::Arc};

use server::http::v1::error::ErrorBody;

use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{
    dto::{
        cluster::{
            ClusterBoundaryListResponse, ClusterBoundaryResponse, ClusterListParams,
            ClusterMarkerListResponse, ClusterMarkerResponse, ClusterStatisticsResponse,
            SoilMoistureParams, SoilMoistureSeriesResponse, TreeClusterCreateRequest,
            TreeClusterInListResponse, TreeClusterResponse, TreeClusterUpdateRequest,
        },
        tree::TreeResponse,
    },
    state::IrrigationHttpState,
};
use domain::{
    Id,
    authorization::{Action, Permission},
    region::Region,
    shared::{
        pagination::Pagination,
        provenance::{Provenance, ProviderId},
        sort::SortDirection,
    },
};
use irrigation_domain::cluster::{
    ClusterAddress, ClusterName, ClusterSort, ClusterSortField, TreeClusterSearchQuery,
    TreeClusterUpdate, TreeClusterView,
};
use server::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path, Query},
        v1::{
            dto::{ListResponse, TransferRequest, sensor::resolve_sensors_by_str_ids},
            scope,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new()
        .routes(routes!(list_clusters, create_cluster))
        .routes(routes!(cluster_statistics))
        .routes(routes!(list_cluster_markers))
        .routes(routes!(list_cluster_boundaries))
        .routes(routes!(get_cluster, update_cluster, delete_cluster))
        .routes(routes!(get_cluster_soil_moisture))
        .routes(routes!(list_cluster_sensor_ids))
        .routes(routes!(transfer_cluster))
}

async fn build_cluster_response(
    state: &IrrigationHttpState,
    cluster: &TreeClusterView,
) -> Result<TreeClusterResponse, ServiceError> {
    let region = match cluster.region_id {
        Some(id) => Some(state.regions.by_id(Id::new(id)).await?),
        None => None,
    };

    let tree_ids: Vec<Id<irrigation_domain::tree::Tree>> =
        cluster.tree_ids.iter().map(|&id| Id::new(id)).collect();
    let trees = state.trees.view_by_ids(&tree_ids).await?;

    let sensor_map = resolve_sensors_by_str_ids(
        &state.sensors,
        trees.iter().filter_map(|t| t.sensor_id.as_deref()),
    )
    .await?;

    let tree_responses: Vec<TreeResponse> = trees
        .iter()
        .map(|t| {
            let sensor = t.sensor_id.as_deref().and_then(|id| sensor_map.get(id));
            TreeResponse::from((t, sensor))
        })
        .collect();

    Ok(TreeClusterResponse::from_parts(
        cluster,
        region.as_ref(),
        tree_responses,
    ))
}

const CLUSTER_LIST_QUERY_MAX_LEN: usize = 100;

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Tree Clusters",
    operation_id = "listClusters",
    summary = "List all tree clusters",
    description = "Returns a paginated list of all tree clusters with a compact representation including region info. \
                   Optional filter parameters (watering_status, region, id) narrow the result; array parameters are repeatable. \
                   Sortable by name, moisture, trees or last_watered.",
    params(ClusterListParams),
    responses(
        (status = 200, description = "Paginated list of tree clusters", body = ListResponse<TreeClusterInListResponse>),
        (status = 400, description = "Invalid query parameter", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_clusters(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Query(params): Query<ClusterListParams>,
) -> Result<Json<ListResponse<TreeClusterInListResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let search = params.query.filter(|s| !s.trim().is_empty());
    if let Some(ref qv) = search
        && qv.chars().count() > CLUSTER_LIST_QUERY_MAX_LEN
    {
        return Err(ServiceError::InvalidInput(format!(
            "query must be at most {CLUSTER_LIST_QUERY_MAX_LEN} characters"
        )));
    }
    let visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        )
        .await?;
    let sort = ClusterSort {
        field: params.sort.map(ClusterSortField::from).unwrap_or_default(),
        direction: params.order.map(SortDirection::from).unwrap_or_default(),
    };
    let query = TreeClusterSearchQuery {
        watering_statuses: params
            .watering_status
            .into_iter()
            .map(irrigation_domain::watering_status::WateringStatus::from)
            .collect(),
        regions: params.region,
        ids: params.id.into_iter().map(Id::new).collect(),
        soil_conditions: params
            .soil_condition
            .into_iter()
            .map(irrigation_domain::cluster::SoilCondition::from)
            .collect(),
        query: search,
        sort,
        visible,
        ..TreeClusterSearchQuery::default()
    };
    let result = state.clusters.search_view(query, pagination).await?;

    let region_ids: Vec<Id<Region>> = result
        .page
        .items
        .iter()
        .filter_map(|c| c.region_id.map(Id::new))
        .collect();
    let regions = state.regions.by_ids(&region_ids).await?;
    let region_map: HashMap<Id<Region>, &_> = regions.iter().map(|r| (r.id, r)).collect();

    let response =
        ListResponse::from_page_with(result.page, &pagination, |cluster: &TreeClusterView| {
            let region = cluster
                .region_id
                .map(Id::new)
                .and_then(|id| region_map.get(&id).copied());
            TreeClusterInListResponse::from((cluster, region))
        })
        .with_total_unfiltered(result.total_unfiltered);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/clusters/{cluster_id}",
    tag = "Tree Clusters",
    operation_id = "getCluster",
    summary = "Get a tree cluster",
    description = "Returns a single tree cluster by its unique identifier, including fully resolved tree objects.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    responses(
        (status = 200, description = "Cluster found", body = TreeClusterResponse),
        (status = 404, description = "Cluster not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn get_cluster(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let view = state.clusters.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        view.organization_id,
    )?;
    let response = build_cluster_response(&state, &view).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/clusters",
    tag = "Tree Clusters",
    operation_id = "createCluster",
    summary = "Create a tree cluster",
    description = "Creates a new tree cluster with the given properties and returns the created resource.",
    request_body = TreeClusterCreateRequest,
    responses(
        (status = 201, description = "Cluster created", body = TreeClusterResponse),
        (status = 422, description = "A selected tree belongs to a different organization (code `organization_mismatch.trees_vs_cluster`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_cluster(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Json(entity): Json<TreeClusterCreateRequest>,
) -> Result<(StatusCode, Json<TreeClusterResponse>), ServiceError> {
    let org = scope::resolve_target_org(
        &state.users,
        &state.authorization,
        user.id,
        entity.organization_id,
    )
    .await?;
    state
        .authorization
        .require(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Create),
            org,
        )
        .await?;
    let draft = entity.into_draft(org)?;
    let cluster = state.clusters.create(draft).await?;
    let view = state.clusters.view_by_id(cluster.id).await?;
    let response = build_cluster_response(&state, &view).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    put,
    path = "/clusters/{cluster_id}",
    tag = "Tree Clusters",
    operation_id = "updateCluster",
    summary = "Update a tree cluster",
    description = "Updates an existing tree cluster identified by its ID and returns the updated resource.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    request_body = TreeClusterUpdateRequest,
    responses(
        (status = 200, description = "Cluster updated", body = TreeClusterResponse),
        (status = 403, description = "Missing tree_cluster:update in the owning organization", body = ErrorBody),
        (status = 404, description = "Cluster not found", body = ErrorBody),
        (status = 422, description = "A selected tree belongs to a different organization (code `organization_mismatch.trees_vs_cluster`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn update_cluster(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<TreeClusterUpdateRequest>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let cluster_id = Id::new(id);
    let current = state.clusters.view_by_id(cluster_id).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        current.organization_id,
    )?;
    state
        .authorization
        .require(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Update),
            Id::new(current.organization_id),
        )
        .await?;
    let update = TreeClusterUpdate {
        name: ClusterName::new(entity.name)?,
        address: ClusterAddress::new(entity.address)?,
        description: entity.description,
        soil_condition: Some(entity.soil_condition.into()),
        tree_ids: entity.tree_ids.into_iter().map(Id::new).collect(),
        provenance: Provenance::new(
            entity.provider.map(ProviderId::new).transpose()?,
            entity.additional_information,
        ),
    };
    state.clusters.replace(cluster_id, update).await?;
    let view = state.clusters.view_by_id(cluster_id).await?;
    let response = build_cluster_response(&state, &view).await?;
    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/clusters/{cluster_id}",
    tag = "Tree Clusters",
    operation_id = "deleteCluster",
    summary = "Delete a tree cluster",
    description = "Deletes a tree cluster identified by its ID.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    responses(
        (status = 204, description = "Cluster deleted"),
        (status = 403, description = "Missing tree_cluster:delete in the owning organization", body = ErrorBody),
        (status = 404, description = "Cluster not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn delete_cluster(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let current = state.clusters.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        current.organization_id,
    )?;
    state
        .authorization
        .require(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Delete),
            Id::new(current.organization_id),
        )
        .await?;
    state.clusters.delete(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/clusters/markers",
    tag = "Tree Clusters",
    operation_id = "listClusterMarkers",
    summary = "List cluster markers",
    description = "Returns lightweight markers (id, name, lat, lng, status, tree_count) \
                   for all non-archived clusters with a centroid. Not paginated.",
    responses(
        (status = 200, description = "Marker list", body = ClusterMarkerListResponse),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_cluster_markers(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
) -> Result<Json<ClusterMarkerListResponse>, ServiceError> {
    let visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        )
        .await?;
    let markers = state.clusters.view_markers(visible).await?;
    let data = markers.iter().map(ClusterMarkerResponse::from).collect();
    Ok(Json(ClusterMarkerListResponse { data }))
}

#[utoipa::path(
    get,
    path = "/clusters/boundaries",
    tag = "Tree Clusters",
    operation_id = "listClusterBoundaries",
    summary = "List cluster boundaries",
    description = "Returns a convex-hull boundary polygon (GeoJSON, buffered by a fixed margin in meters) \
                   around the trees of each non-archived cluster. Not paginated.",
    responses(
        (status = 200, description = "Boundary list", body = ClusterBoundaryListResponse),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_cluster_boundaries(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
) -> Result<Json<ClusterBoundaryListResponse>, ServiceError> {
    let visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        )
        .await?;
    let boundaries = state.clusters.boundaries(visible).await?;
    let data = boundaries
        .iter()
        .map(ClusterBoundaryResponse::from)
        .collect();
    Ok(Json(ClusterBoundaryListResponse { data }))
}

#[utoipa::path(
    get,
    path = "/clusters/statistics",
    tag = "Tree Clusters",
    operation_id = "getClusterStatistics",
    summary = "Cluster statistics",
    description = "Counts of non-archived clusters per watering status, plus total clusters and total trees in clusters.",
    responses(
        (status = 200, description = "Cluster statistics", body = ClusterStatisticsResponse),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn cluster_statistics(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
) -> Result<Json<ClusterStatisticsResponse>, ServiceError> {
    let visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        )
        .await?;
    let stats = state.clusters.statistics(visible).await?;
    Ok(Json(ClusterStatisticsResponse::from(stats)))
}

#[utoipa::path(
    get,
    path = "/clusters/{cluster_id}/soil-moisture",
    tag = "Tree Clusters",
    operation_id = "getClusterSoilMoisture",
    summary = "Bucketed soil-moisture series for a cluster",
    description = "Aggregates volumetric soil-moisture readings (mean/min/max per probe depth and time bucket) \
                   across all sensors of the cluster, with per-depth stress thresholds derived from the \
                   cluster's soil condition and the cluster's finished watering runs.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID"), SoilMoistureParams),
    responses(
        (status = 200, description = "Aggregated soil-moisture series", body = SoilMoistureSeriesResponse),
        (status = 400, description = "Invalid query parameter", body = ErrorBody),
        (status = 404, description = "Cluster not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn get_cluster_soil_moisture(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Query(params): Query<SoilMoistureParams>,
) -> Result<Json<SoilMoistureSeriesResponse>, ServiceError> {
    let cluster_id = Id::new(id);
    let view = state.clusters.view_by_id(cluster_id).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        view.organization_id,
    )?;
    let (from, to, bucket) = params.resolve()?;
    let overview = state
        .clusters
        .soil_moisture_overview(cluster_id, from, to, bucket)
        .await?;
    Ok(Json(SoilMoistureSeriesResponse::from(overview)))
}

#[utoipa::path(
    get,
    path = "/clusters/{cluster_id}/sensor-ids",
    tag = "Tree Clusters",
    operation_id = "listClusterSensorIds",
    summary = "List the sensors of a cluster",
    description = "Returns the ids of the sensors attached to the cluster's trees, ordered by id. \
                   Feed them into `GET /sensors?id=…` to filter the sensor list by cluster.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    responses(
        (status = 200, description = "Sensor ids", body = Vec<String>),
        (status = 404, description = "Cluster not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn list_cluster_sensor_ids(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Vec<String>>, ServiceError> {
    let cluster_id = Id::new(id);
    let view = state.clusters.view_by_id(cluster_id).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        view.organization_id,
    )?;
    let ids = state.clusters.sensor_ids(cluster_id).await?;
    Ok(Json(ids.iter().map(|id| id.as_str().to_owned()).collect()))
}

#[utoipa::path(
    patch,
    path = "/clusters/{cluster_id}/organization",
    tag = "Tree Clusters",
    operation_id = "transferCluster",
    summary = "Transfer a cluster's ownership to another organization",
    description = "Moves the cluster, its member trees, and their attached sensors to a \
                   different owning organization in one operation. Requires `tree_cluster:update` \
                   in both the source and target organization.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    request_body = TransferRequest,
    responses(
        (status = 204, description = "Cluster transferred"),
        (status = 403, description = "Missing tree_cluster:update in source or target organization", body = ErrorBody),
        (status = 404, description = "Cluster or organization not found", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %id))]
pub async fn transfer_cluster(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = state.clusters.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        current.organization_id,
    )?;
    let perm = Permission::new(crate::resources::TREE_CLUSTER, Action::Update);
    state
        .authorization
        .require(user.id, perm, Id::new(current.organization_id))
        .await?;
    state
        .authorization
        .require(user.id, perm, Id::new(req.organization_id))
        .await?;
    state
        .clusters
        .transfer(Id::new(id), Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
