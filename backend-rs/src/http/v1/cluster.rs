use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

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

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/clusters", get(list_clusters).post(create_cluster))
        .route(
            "/clusters/{cluster_id}",
            get(get_cluster).put(update_cluster).delete(delete_cluster),
        )
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

pub async fn list_clusters(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeClusterInListResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .cluster_service
        .all(TreeClusterQuery::default(), pagination)
        .await?;

    let region_ids: Vec<_> = page.items.iter().filter_map(|c| c.region_id).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();

    let response = ListResponse::from_page_with(page, params.page, params.per_page, |cluster| {
        let region = cluster
            .region_id
            .and_then(|id| region_map.get(&id).copied());
        TreeClusterInListResponse::from((cluster, region))
    });
    Ok(Json(response))
}

pub async fn get_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeClusterResponse>, ServiceError> {
    let cluster = state.cluster_service.by_id(Id::from(id)).await?;
    let response = build_cluster_response(&state, &cluster).await?;
    Ok(Json(response))
}

pub async fn create_cluster(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<TreeClusterCreateRequest>,
) -> Result<(StatusCode, Json<TreeClusterResponse>), ServiceError> {
    let cluster = state.cluster_service.create(entity.into()).await?;
    let response = build_cluster_response(&state, &cluster).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

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

pub async fn delete_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.cluster_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
