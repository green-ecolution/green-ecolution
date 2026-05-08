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
                cluster::{
                    ClusterMarkerListResponse, ClusterMarkerResponse, TreeClusterCreateRequest,
                    TreeClusterInListResponse, TreeClusterResponse, TreeClusterUpdateRequest,
                },
                sensor::resolve_sensors_by_str_ids,
                tree::TreeResponse,
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    cluster::{
        ClusterAddress, ClusterName, TreeClusterDraft, TreeClusterSearchQuery, TreeClusterUpdate,
        TreeClusterView,
    },
    region::Region,
    shared::{
        pagination::Pagination,
        provenance::{Provenance, ProviderId},
    },
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_clusters, create_cluster))
        .routes(routes!(list_cluster_markers))
        .routes(routes!(get_cluster, update_cluster, delete_cluster))
}

async fn build_cluster_response(
    state: &AppState,
    cluster: &TreeClusterView,
) -> Result<TreeClusterResponse, ServiceError> {
    let region = match cluster.region_id {
        Some(id) => Some(state.region_service.by_id(Id::new(id)).await?),
        None => None,
    };

    let tree_ids: Vec<Id<domain::tree::Tree>> =
        cluster.tree_ids.iter().map(|&id| Id::new(id)).collect();
    let trees = state.tree_service.view_by_ids(&tree_ids).await?;

    let sensor_map = resolve_sensors_by_str_ids(
        &state.sensor_service,
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

#[utoipa::path(
    get,
    path = "/clusters",
    tag = "Tree Clusters",
    operation_id = "listClusters",
    summary = "List all tree clusters",
    description = "Returns a paginated list of all tree clusters with a compact representation including region info.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of tree clusters", body = ListResponse<TreeClusterInListResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_clusters(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeClusterInListResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .cluster_service
        .search_view(TreeClusterSearchQuery::default(), pagination)
        .await?;

    let region_ids: Vec<Id<Region>> = page
        .items
        .iter()
        .filter_map(|c| c.region_id.map(Id::new))
        .collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<Id<Region>, &_> = regions.iter().map(|r| (r.id, r)).collect();

    let response = ListResponse::from_page_with(page, &pagination, |cluster: &TreeClusterView| {
        let region = cluster
            .region_id
            .map(Id::new)
            .and_then(|id| region_map.get(&id).copied());
        TreeClusterInListResponse::from((cluster, region))
    });
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/clusters/{cluster_id}",
    tag = "Tree Clusters",
    operation_id = "getCluster",
    summary = "Get a tree cluster",
    description = "Returns a single tree cluster by its unique identifier, including fully resolved tree objects.",
    params(("cluster_id" = i32, Path, description = "Cluster ID")),
    responses(
        (status = 200, description = "Cluster found", body = TreeClusterResponse),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = id))]
pub async fn get_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let view = state.cluster_service.view_by_id(Id::from(id)).await?;
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
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_cluster(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<TreeClusterCreateRequest>,
) -> Result<(StatusCode, Json<TreeClusterResponse>), ServiceError> {
    let draft: TreeClusterDraft = entity.try_into()?;
    let cluster = state.cluster_service.create(draft).await?;
    let view = state.cluster_service.view_by_id(cluster.id).await?;
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
    params(("cluster_id" = i32, Path, description = "Cluster ID")),
    request_body = TreeClusterUpdateRequest,
    responses(
        (status = 200, description = "Cluster updated", body = TreeClusterResponse),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = id))]
pub async fn update_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<TreeClusterUpdateRequest>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let cluster_id = Id::from(id);
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
    state.cluster_service.replace(cluster_id, update).await?;
    let view = state.cluster_service.view_by_id(cluster_id).await?;
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
    params(("cluster_id" = i32, Path, description = "Cluster ID")),
    responses(
        (status = 204, description = "Cluster deleted"),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = id))]
pub async fn delete_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.cluster_service.delete(Id::from(id)).await?;
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
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_cluster_markers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ClusterMarkerListResponse>, ServiceError> {
    let markers = state.cluster_service.view_markers().await?;
    let data = markers.iter().map(ClusterMarkerResponse::from).collect();
    Ok(Json(ClusterMarkerListResponse { data }))
}
