use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{sensor::SensorQuery, shared::pagination::Pagination},
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
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_sensors))
        .routes(routes!(get_sensor, delete_sensor))
        .routes(routes!(list_sensor_data))
}

#[utoipa::path(get, path = "/sensors", tag = "Sensors",
    params(PaginationParams),
    responses((status = 200, description = "Paginated list of sensors"))
)]
pub async fn list_sensors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<SensorResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .sensor_service
        .all(SensorQuery::default(), pagination)
        .await?;
    let response = ListResponse::<SensorResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

#[utoipa::path(get, path = "/sensors/{sensor_id}", tag = "Sensors",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 200, description = "Sensor found", body = SensorResponse),
        (status = 404, description = "Sensor not found"),
    )
)]
pub async fn get_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.by_id(&id).await?;
    Ok(Json(SensorResponse::from(&sensor)))
}

#[utoipa::path(delete, path = "/sensors/{sensor_id}", tag = "Sensors",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 204, description = "Sensor deleted"),
        (status = 404, description = "Sensor not found"),
    )
)]
pub async fn delete_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    state.sensor_service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/data", tag = "Sensors",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses((status = 200, description = "Sensor data list"))
)]
pub async fn list_sensor_data(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SensorDataResponse>>, ServiceError> {
    let data = state.sensor_service.all_data(&id).await?;
    let response: Vec<SensorDataResponse> = data.iter().map(SensorDataResponse::from).collect();
    Ok(Json(response))
}
