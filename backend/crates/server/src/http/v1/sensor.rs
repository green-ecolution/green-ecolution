use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        extractors::SensorIdPath,
        v1::{
            dto::{
                ListResponse,
                sensor::{
                    ActivateSensorRequest, CreateSensorRequest, LorawanCredentialsRequest,
                    SensorDataResponse, SensorModelResponse, SensorResponse, SetSensorTreeRequest,
                },
                tree::TreeResponse,
            },
            pagination::{PaginationParams, default_page},
        },
    },
    service::ServiceError,
};
use domain::{
    Id, RepositoryError,
    sensor::{LorawanCredentials, SensorDraft, SensorId, SensorSearchQuery, SensorType},
    sensor_model::SensorModel,
    shared::{
        pagination::Pagination,
        provenance::{Provenance, ProviderId},
        string_value::NonEmptyString,
    },
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_sensors, create_sensor))
        .routes(routes!(get_sensor, delete_sensor))
        .routes(routes!(activate_sensor))
        .routes(routes!(list_sensor_data))
        .routes(routes!(
            get_tree_by_sensor,
            set_sensor_tree,
            remove_sensor_tree
        ))
        .routes(routes!(list_sensor_models))
        .routes(routes!(get_sensor_model))
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

/// Charts need whole time windows, so the readings page-size ceiling is far
/// above the list default of 100.
const SENSOR_DATA_DEFAULT_PER_PAGE: u64 = 500;
const SENSOR_DATA_MAX_PER_PAGE: u64 = 5_000;

fn default_sensor_data_per_page() -> u64 {
    SENSOR_DATA_DEFAULT_PER_PAGE
}

/// Query parameters for the sensor readings endpoint.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct SensorDataParams {
    /// Page number to retrieve (1-based).
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "default_page")]
    pub page: u64,

    /// Number of readings per page.
    #[param(default = 500, minimum = 1, maximum = 5000, example = 500)]
    #[serde(default = "default_sensor_data_per_page")]
    pub per_page: u64,

    /// Only readings recorded at or after this timestamp (RFC 3339).
    #[param(value_type = Option<String>, format = DateTime, example = "2026-07-06T00:00:00Z")]
    pub from: Option<DateTime<Utc>>,

    /// Only readings recorded at or before this timestamp (RFC 3339).
    #[param(value_type = Option<String>, format = DateTime, example = "2026-07-13T00:00:00Z")]
    pub to: Option<DateTime<Utc>>,
}

#[utoipa::path(get, path = "/sensors/{sensor_id}/data", tag = "Sensors",
    operation_id = "listSensorData",
    summary = "List sensor data",
    description = "Returns a paginated list of historical data readings for a sensor, \
        optionally restricted to a time range.",
    params(("sensor_id" = String, Path, description = "Sensor ID"), SensorDataParams),
    responses(
        (status = 200, description = "Paginated sensor data", body = ListResponse<SensorDataResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn list_sensor_data(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
    Query(params): Query<SensorDataParams>,
) -> Result<Json<ListResponse<SensorDataResponse>>, ServiceError> {
    let pagination =
        Pagination::with_max_per_page(params.page, params.per_page, SENSOR_DATA_MAX_PER_PAGE);
    let page = state
        .sensor_service
        .view_history(&sensor_id, pagination, params.from, params.to)
        .await?;
    Ok(Json(ListResponse::from_page(page, &pagination)))
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

#[utoipa::path(post, path = "/sensors", tag = "Sensors",
    operation_id = "createSensor",
    summary = "Register a new (prepared) sensor unit",
    description = "Creates a sensor record in `Prepared` state. The sensor must later be \
        bound to a tree via `POST /sensors/{sensor_id}/activate` before it starts \
        receiving data.",
    request_body = CreateSensorRequest,
    responses(
        (status = 201, description = "Sensor created", body = SensorResponse),
        (status = 400, description = "Invalid request body"),
        (status = 404, description = "Sensor model not found"),
        (status = 409, description = "Sensor id already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_sensor(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSensorRequest>,
) -> Result<(StatusCode, Json<SensorResponse>), ServiceError> {
    let sensor_type: SensorType = body.sensor_type.into();
    let lorawan = match (sensor_type, body.lorawan) {
        (SensorType::Lorawan, Some(l)) => parse_lorawan(l)?,
        (SensorType::Lorawan, None) => {
            return Err(ServiceError::InvalidInput(
                "lorawan block required for sensor_type=lorawan".into(),
            ));
        }
    };

    let draft = SensorDraft {
        id: SensorId::new(body.id)?,
        sensor_type,
        model_id: Id::new(body.model_id),
        provenance: Provenance::new(
            body.provider.map(ProviderId::new).transpose()?,
            body.additional_information,
        ),
        lorawan,
    };
    let view = state.sensor_service.create(draft).await?;
    Ok((StatusCode::CREATED, Json(SensorResponse::from(&view))))
}

#[utoipa::path(post, path = "/sensors/{sensor_id}/activate", tag = "Sensors",
    operation_id = "activateSensor",
    summary = "Activate a prepared sensor by binding it to a tree",
    description = "Transitions a sensor from `Prepared` to `Offline` and attaches it to \
        the given tree. Idempotent if the sensor is already attached to the same tree.",
    params(("sensor_id" = String, Path, description = "Sensor ID (EUI)")),
    request_body = ActivateSensorRequest,
    responses(
        (status = 200, description = "Sensor activated", body = SensorResponse),
        (status = 404, description = "Sensor or tree not found"),
        (status = 409, description = "Conflict: sensor or tree already linked"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn activate_sensor(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
    Json(body): Json<ActivateSensorRequest>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let view = state
        .sensor_service
        .activate(&sensor_id, Id::new(body.tree_id))
        .await?;
    Ok(Json(SensorResponse::from(&view)))
}

#[utoipa::path(put, path = "/sensors/{sensor_id}/tree", tag = "Sensors",
    operation_id = "setSensorTree",
    summary = "Move an activated sensor to a different tree",
    description = "Re-links an already activated sensor to `tree_id`. Rejects a \
        tree that already has a different sensor and a sensor that is not yet \
        activated. Idempotent if the sensor is already linked to that tree.",
    params(("sensor_id" = String, Path, description = "Sensor ID (EUI)")),
    request_body = SetSensorTreeRequest,
    responses(
        (status = 200, description = "Sensor re-linked", body = SensorResponse),
        (status = 404, description = "Sensor or tree not found"),
        (status = 409, description = "Conflict: sensor not activated or tree already linked"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn set_sensor_tree(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
    Json(body): Json<SetSensorTreeRequest>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let view = state
        .sensor_service
        .reassign_tree(&sensor_id, Id::new(body.tree_id))
        .await?;
    Ok(Json(SensorResponse::from(&view)))
}

#[utoipa::path(delete, path = "/sensors/{sensor_id}/tree", tag = "Sensors",
    operation_id = "removeSensorTree",
    summary = "Remove a sensor's tree link and reset it to prepared",
    description = "Detaches the sensor from its tree and deactivates it, \
        returning it to the `Prepared` state. Idempotent for an already \
        prepared sensor.",
    params(("sensor_id" = String, Path, description = "Sensor ID (EUI)")),
    responses(
        (status = 200, description = "Sensor reset to prepared", body = SensorResponse),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn remove_sensor_tree(
    State(state): State<Arc<AppState>>,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<SensorResponse>, ServiceError> {
    let view = state.sensor_service.deactivate(&sensor_id).await?;
    Ok(Json(SensorResponse::from(&view)))
}

#[utoipa::path(get, path = "/sensors/models", tag = "Sensors",
    operation_id = "listSensorModels",
    summary = "List all supported sensor models",
    description = "Returns every sensor model registered in the catalogue along with \
        its abilities (e.g. soil tension at 30/60/90 cm).",
    responses(
        (status = 200, description = "Sensor models", body = Vec<SensorModelResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_sensor_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SensorModelResponse>>, ServiceError> {
    let models = state.sensor_service.list_models().await?;
    Ok(Json(models.iter().map(SensorModelResponse::from).collect()))
}

#[utoipa::path(get, path = "/sensors/models/{id}", tag = "Sensors",
    operation_id = "getSensorModel",
    summary = "Get a single sensor model",
    params(("id" = uuid::Uuid, Path, description = "Sensor model ID")),
    responses(
        (status = 200, description = "Sensor model", body = SensorModelResponse),
        (status = 404, description = "Sensor model not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(model.id = %id))]
pub async fn get_sensor_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<SensorModelResponse>, ServiceError> {
    let model = state
        .sensor_service
        .model_by_id(Id::<SensorModel>::new(id))
        .await?;
    Ok(Json(SensorModelResponse::from(&model)))
}

fn parse_lorawan(l: LorawanCredentialsRequest) -> Result<LorawanCredentials, ServiceError> {
    hex_field("dev_eui", &l.dev_eui, 16)?;
    hex_field("app_eui", &l.app_eui, 16)?;
    hex_field("app_key", &l.app_key, 32)?;

    Ok(LorawanCredentials {
        serial_number: NonEmptyString::new(l.serial_number, "sensor.lorawan.serial_number", 1, 64)?,
        dev_eui: NonEmptyString::new(l.dev_eui, "sensor.lorawan.dev_eui", 16, 16)?,
        app_eui: NonEmptyString::new(l.app_eui, "sensor.lorawan.app_eui", 16, 16)?,
        app_key: SecretString::from(l.app_key),
        at_pin: l.at_pin,
        ota_pin: l.ota_pin,
        config: l.config,
    })
}

fn hex_field(label: &'static str, s: &str, len: usize) -> Result<(), ServiceError> {
    if s.len() != len || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ServiceError::InvalidInput(format!(
            "{label} must be {len} hex characters"
        )));
    }
    Ok(())
}
