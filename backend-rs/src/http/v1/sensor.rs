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
    operation_id = "listSensors",
    summary = "List all sensors",
    description = "Returns a paginated list of all LoRaWAN sensors.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of sensors", body = ListResponse<SensorResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_sensors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<SensorResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .sensor_service
        .all(SensorQuery::default(), pagination)
        .await?;
    let response = ListResponse::<SensorResponse>::from_page(page, &pagination);
    Ok(Json(response))
}

#[utoipa::path(get, path = "/sensors/{sensor_id}", tag = "Sensors",
    operation_id = "getSensor",
    summary = "Get a sensor by ID",
    description = "Returns a single sensor by its EUI identifier.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 200, description = "Sensor found", body = SensorResponse),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %id))]
pub async fn get_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.by_id(&id).await?;
    Ok(Json(SensorResponse::from(&sensor)))
}

#[utoipa::path(delete, path = "/sensors/{sensor_id}", tag = "Sensors",
    operation_id = "deleteSensor",
    summary = "Delete a sensor",
    description = "Permanently deletes a sensor by its EUI identifier.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 204, description = "Sensor deleted"),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %id))]
pub async fn delete_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    state.sensor_service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/data", tag = "Sensors",
    operation_id = "listSensorData",
    summary = "List sensor data",
    description = "Returns all historical data readings for a sensor.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 200, description = "Sensor data list", body = Vec<SensorDataResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %id))]
pub async fn list_sensor_data(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SensorDataResponse>>, ServiceError> {
    let data = state.sensor_service.all_data(&id).await?;
    let response: Vec<SensorDataResponse> = data.iter().map(SensorDataResponse::from).collect();
    Ok(Json(response))
}
