use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/clusters", get(list_clusters).post(create_cluster))
        .route(
            "/clusters/{cluster_id}",
            get(get_cluster).put(update_cluster).delete(delete_cluster),
        )
}

pub async fn list_clusters(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_cluster(State(_state): State<Arc<AppState>>, Path(_id): Path<i32>) -> Json<()> {
    todo!()
}

pub async fn create_cluster(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn update_cluster(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn delete_cluster(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}
