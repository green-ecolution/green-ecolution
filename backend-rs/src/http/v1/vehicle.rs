use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/vehicles", get(list_vehicles).post(create_vehicle))
        .route("/vehicles/archived", get(list_archived_vehicles))
        .route("/vehicles/archived/{vehicle_id}", post(archive_vehicle))
        .route("/vehicles/plate/{plate}", get(get_vehicle_by_plate))
        .route(
            "/vehicles/{vehicle_id}",
            get(get_vehicle).put(update_vehicle).delete(delete_vehicle),
        )
}

pub async fn list_vehicles(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_vehicle(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn create_vehicle(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn update_vehicle(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn delete_vehicle(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn list_archived_vehicles(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn archive_vehicle(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn get_vehicle_by_plate(
    State(_state): State<Arc<AppState>>,
    Path(_plate): Path<String>,
) -> Json<()> {
    todo!()
}
