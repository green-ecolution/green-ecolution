use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sensors", get(list_sensors))
        .route(
            "/sensors/{sensor_id}",
            get(get_sensor).delete(delete_sensor),
        )
        .route("/sensors/{sensor_id}/data", get(list_sensor_data))
}

pub async fn list_sensors(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_sensor(State(_state): State<Arc<AppState>>, Path(_id): Path<i32>) -> Json<()> {
    todo!()
}

pub async fn delete_sensor(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn list_sensor_data(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}
