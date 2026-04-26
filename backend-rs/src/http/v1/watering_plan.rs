use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/watering-plans",
            get(list_watering_plans).post(create_watering_plan),
        )
        .route(
            "/watering-plans/route/gpx/{gpx_name}",
            get(get_gpx_file),
        )
        .route("/watering-plans/route/preview", post(preview_route))
        .route(
            "/watering-plans/{watering_plan_id}",
            get(get_watering_plan)
                .put(update_watering_plan)
                .delete(delete_watering_plan),
        )
}

pub async fn list_watering_plans(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn get_watering_plan(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn create_watering_plan(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn update_watering_plan(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn delete_watering_plan(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<i32>,
) -> Json<()> {
    todo!()
}

pub async fn get_gpx_file(
    State(_state): State<Arc<AppState>>,
    Path(_name): Path<String>,
) -> Json<()> {
    todo!()
}

pub async fn preview_route(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}
