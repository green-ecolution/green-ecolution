use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{Id, cluster::TreeCluster, cluster::TreeClusterQuery, shared::pagination::Pagination},
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                cluster::{
                    TreeClusterCreateRequest, TreeClusterInListResponse, TreeClusterResponse,
                    TreeClusterUpdateRequest, TreeClusterView,
                },
                tree::TreeResponse,
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_clusters, create_cluster))
        .routes(routes!(get_cluster, update_cluster, delete_cluster))
}

async fn build_cluster_response(
    state: &AppState,
    cluster: &TreeCluster,
) -> Result<TreeClusterResponse, ServiceError> {
    let region = match cluster.region_id {
        Some(id) => Some(state.region_service.by_id(id).await?),
        None => None,
    };

    let trees = state.tree_service.by_ids(&cluster.tree_ids).await?;

    let sensor_ids: Vec<String> = trees.iter().filter_map(|t| t.sensor_id.clone()).collect();
    let sensors = state.sensor_service.by_ids(&sensor_ids).await?;
    let sensor_map: HashMap<&str, _> = sensors.iter().map(|s| (s.id.as_str(), s)).collect();

    let tree_responses: Vec<TreeResponse> = trees
        .iter()
        .map(|t| {
            let sensor = t
                .sensor_id
                .as_deref()
                .and_then(|id| sensor_map.get(id).copied());
            TreeResponse::from((t, sensor))
        })
        .collect();

    Ok(TreeClusterResponse::from(TreeClusterView {
        cluster,
        region: region.as_ref(),
        trees: tree_responses,
    }))
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
pub async fn list_clusters(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeClusterInListResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .cluster_service
        .all(TreeClusterQuery::default(), pagination)
        .await?;

    let region_ids: Vec<_> = page.items.iter().filter_map(|c| c.region_id).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();

    let response = ListResponse::from_page_with(page, &pagination, |cluster: &TreeCluster| {
        let region = cluster
            .region_id
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
pub async fn get_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let cluster = state.cluster_service.by_id(Id::from(id)).await?;
    let response = build_cluster_response(&state, &cluster).await?;
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
pub async fn create_cluster(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<TreeClusterCreateRequest>,
) -> Result<(StatusCode, Json<TreeClusterResponse>), ServiceError> {
    let cluster = state.cluster_service.create(entity.into()).await?;
    let response = build_cluster_response(&state, &cluster).await?;
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
pub async fn update_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<TreeClusterUpdateRequest>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let cluster_id = Id::from(id);
    state
        .cluster_service
        .update(cluster_id, entity.into())
        .await?;
    let cluster = state.cluster_service.by_id(cluster_id).await?;
    let response = build_cluster_response(&state, &cluster).await?;
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
pub async fn delete_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.cluster_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
