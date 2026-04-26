use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

use crate::{
    domain::sensor::SensorQuery,
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                sensor::{SensorDataResponse, SensorResponse},
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
    domain::shared::pagination::Pagination,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sensors", get(list_sensors))
        .route(
            "/sensors/{sensor_id}",
            get(get_sensor).delete(delete_sensor),
        )
        .route("/sensors/{sensor_id}/data", get(list_sensor_data))
}

pub async fn list_sensors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<SensorResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .sensor_service
        .all(SensorQuery::default(), pagination)
        .await?;
    let response =
        ListResponse::<SensorResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

pub async fn get_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.by_id(&id).await?;
    Ok(Json(SensorResponse::from(&sensor)))
}

pub async fn delete_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    state.sensor_service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_sensor_data(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SensorDataResponse>>, ServiceError> {
    let data = state.sensor_service.all_data(&id).await?;
    let response: Vec<SensorDataResponse> = data.iter().map(SensorDataResponse::from).collect();
    Ok(Json(response))
}
