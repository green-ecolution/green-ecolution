use std::sync::Arc;

use axum::extract::State;
use domain::{
    RepositoryError,
    authorization::{Action, Permission, ResourceKey},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{
    dto::{
        cluster::{SoilMoistureParams, SoilMoistureSeriesResponse},
        tree::TreeResponse,
    },
    state::IrrigationHttpState,
};
use server::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Query, SensorIdPath},
        v1::{error::ErrorBody, scope},
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new()
        .routes(routes!(get_sensor_soil_moisture))
        .routes(routes!(get_tree_by_sensor))
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/soil-moisture", tag = "Sensors",
    operation_id = "getSensorSoilMoisture",
    summary = "Bucketed soil-moisture series for a sensor",
    description = "Aggregates the sensor's volumetric soil-moisture readings (mean/min/max per probe \
        depth and time bucket). Stress thresholds and the REW condition series are derived from the \
        soil condition of the linked tree's cluster and are empty when the sensor is not linked or \
        the soil is unknown.",
    params(("sensor_id" = String, Path, description = "Sensor ID"), SoilMoistureParams),
    responses(
        (status = 200, description = "Aggregated soil-moisture series", body = SoilMoistureSeriesResponse),
        (status = 400, description = "Invalid query parameter", body = ErrorBody),
        (status = 404, description = "Sensor not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_sensor_soil_moisture(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Query(params): Query<SoilMoistureParams>,
) -> Result<Json<SoilMoistureSeriesResponse>, ServiceError> {
    let sensor = state.sensors.view_by_id(&sensor_id).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(ResourceKey::SENSOR, Action::Read),
        sensor.organization_id,
    )?;
    let (from, to, bucket) = params.resolve()?;
    let overview = state
        .sensor_overview
        .soil_moisture_overview(&sensor_id, from, to, bucket)
        .await?;
    Ok(Json(SoilMoistureSeriesResponse::from(overview)))
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/tree", tag = "Trees",
    operation_id = "getTreeBySensor",
    summary = "Get the tree associated with a sensor",
    description = "Retrieves the tree linked to the given sensor. Returns 404 if the sensor or its associated tree does not exist.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Sensor or associated tree not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_tree_by_sensor(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<TreeResponse>, ServiceError> {
    let sensor = state.sensors.view_by_id(&sensor_id).await?;
    let ctx = state.authorization.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(ResourceKey::SENSOR, Action::Read),
        sensor.organization_id,
    )?;
    let tree = state
        .trees
        .view_by_sensor_id(&sensor_id)
        .await?
        .ok_or(ServiceError::Repository(RepositoryError::NotFound))?;
    Ok(Json(TreeResponse::from((&tree, Some(&sensor)))))
}
