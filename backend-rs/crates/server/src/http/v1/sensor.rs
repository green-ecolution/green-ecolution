use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        extractors::SensorIdPath,
        v1::{
            dto::{
                ListResponse,
                sensor::{SensorDataResponse, SensorResponse},
                tree::TreeResponse,
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};
use domain::{RepositoryError, sensor::SensorSearchQuery, shared::pagination::Pagination};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_sensors))
        .routes(routes!(get_sensor, delete_sensor))
        .routes(routes!(list_sensor_data))
        .routes(routes!(get_tree_by_sensor))
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
        .search_view(SensorSearchQuery::default(), pagination)
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
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_sensor(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<SensorResponse>, ServiceError> {
    let view = state.sensor_service.view_by_id(&sensor_id).await?;
    Ok(Json(SensorResponse::from(&view)))
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
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn delete_sensor(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<StatusCode, ServiceError> {
    state.sensor_service.delete(&sensor_id).await?;
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
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn list_sensor_data(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<Vec<SensorDataResponse>>, ServiceError> {
    let readings = state
        .sensor_service
        .view_history(&sensor_id, 10_000)
        .await?;
    let response: Vec<SensorDataResponse> = readings.iter().map(SensorDataResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/tree", tag = "Trees",
    operation_id = "getTreeBySensor",
    summary = "Get the tree associated with a sensor",
    description = "Retrieves the tree linked to the given sensor. Returns 404 if the sensor or its associated tree does not exist.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Sensor or associated tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_tree_by_sensor(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<TreeResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let tree = state
        .tree_service
        .view_by_sensor_id(&sensor_id)
        .await?
        .ok_or(ServiceError::Repository(RepositoryError::NotFound))?;
    Ok(Json(TreeResponse::from((&tree, Some(&sensor)))))
}
