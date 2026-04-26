use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trees", get(list_trees).post(create_tree))
        .route("/trees/planting-years", get(list_planting_years))
        .route(
            "/trees/{tree_id}",
            get(get_tree).put(update_tree).delete(delete_tree),
        )
        .route(
            "/trees/{tree_id}/sensors/{sensor_id}",
            get(get_tree_sensor),
        )
}

pub async fn list_trees(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_tree(State(_state): State<Arc<AppState>>, Path(_id): Path<i32>) -> Json<()> {
    todo!()
}

pub async fn create_tree(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn update_tree(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn delete_tree(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn list_planting_years(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_tree_sensor(
    State(_state): State<Arc<AppState>>,
    Path((_tree_id, _sensor_id)): Path<(i32, i32)>,
) -> Json<()> {
    todo!()
}
