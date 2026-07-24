use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::SensorIdPath,
        v1::{
            dto::{
                ListResponse,
                cluster::{SoilMoistureParams, SoilMoistureSeriesResponse},
                sensor::{
                    ActivateSensorRequest, CreateSensorRequest, SensorDataResponse,
                    SensorModelResponse, SensorResponse, SetSensorTreeRequest,
                },
                tree::{TransferRequest, TreeResponse},
            },
            pagination::{PaginationParams, default_page},
            scope,
        },
    },
    service::ServiceError,
};
use domain::{
    Id, RepositoryError,
    authorization::{Action, Permission, Resource},
    sensor::SensorSearchQuery,
    sensor_model::SensorModel,
    shared::pagination::Pagination,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_sensors, create_sensor))
        .routes(routes!(get_sensor, delete_sensor))
        .routes(routes!(activate_sensor))
        .routes(routes!(list_sensor_data))
        .routes(routes!(get_sensor_soil_moisture))
        .routes(routes!(
            get_tree_by_sensor,
            set_sensor_tree,
            remove_sensor_tree
        ))
        .routes(routes!(list_sensor_models))
        .routes(routes!(get_sensor_model))
        .routes(routes!(transfer_sensor))
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
    user: AuthUserExtractor,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<SensorResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Sensor, Action::Read))
        .await?;
    let query = SensorSearchQuery {
        visible,
        ..SensorSearchQuery::default()
    };
    let page = state.sensor_service.search_view(query, pagination).await?;
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
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<SensorResponse>, ServiceError> {
    let view = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &scope::effective_orgs(view.organization_id, &view.shared_with),
    )?;
    Ok(Json(SensorResponse::from(&view)))
}

#[utoipa::path(delete, path = "/sensors/{sensor_id}", tag = "Sensors",
    operation_id = "deleteSensor",
    summary = "Delete a sensor",
    description = "Permanently deletes a sensor by its EUI identifier.",
    params(("sensor_id" = String, Path, description = "Sensor ID")),
    responses(
        (status = 204, description = "Sensor deleted"),
        (status = 403, description = "Missing sensor:delete in the owning organization"),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn delete_sensor(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<StatusCode, ServiceError> {
    let current = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &scope::effective_orgs(current.organization_id, &current.shared_with),
    )?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Sensor, Action::Delete),
            Id::new(current.organization_id),
        )
        .await?;
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
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Query(params): Query<SensorDataParams>,
) -> Result<Json<ListResponse<SensorDataResponse>>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &scope::effective_orgs(sensor.organization_id, &sensor.shared_with),
    )?;
    let pagination =
        Pagination::with_max_per_page(params.page, params.per_page, SENSOR_DATA_MAX_PER_PAGE);
    let page = state
        .sensor_service
        .view_history(&sensor_id, pagination, params.from, params.to)
        .await?;
    Ok(Json(ListResponse::from_page(page, &pagination)))
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
        (status = 400, description = "Invalid query parameter"),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_sensor_soil_moisture(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Query(params): Query<SoilMoistureParams>,
) -> Result<Json<SoilMoistureSeriesResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &scope::effective_orgs(sensor.organization_id, &sensor.shared_with),
    )?;
    let (from, to, bucket) = params.resolve()?;
    let overview = state
        .sensor_service
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
        (status = 404, description = "Sensor or associated tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn get_tree_by_sensor(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<TreeResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &scope::effective_orgs(sensor.organization_id, &sensor.shared_with),
    )?;
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
    user: AuthUserExtractor,
    Json(body): Json<CreateSensorRequest>,
) -> Result<(StatusCode, Json<SensorResponse>), ServiceError> {
    let org = scope::resolve_target_org(&state, user.id, body.organization_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Sensor, Action::Create),
            org,
        )
        .await?;
    let draft = body.into_draft(org)?;
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
        (status = 403, description = "Missing sensor:update or tree:update in the effective organizations"),
        (status = 404, description = "Sensor or tree not found"),
        (status = 409, description = "Conflict: sensor or tree already linked"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn activate_sensor(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Json(body): Json<ActivateSensorRequest>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let sensor_orgs = scope::effective_orgs(sensor.organization_id, &sensor.shared_with);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &sensor_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Sensor, Action::Update),
            &sensor_orgs,
        )
        .await?;

    let tree = state.tree_service.view_by_id(Id::new(body.tree_id)).await?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Tree, Action::Update),
            &scope::effective_orgs(tree.organization_id, &tree.shared_with),
        )
        .await?;

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
        (status = 403, description = "Missing sensor:update or tree:update in the effective organizations"),
        (status = 404, description = "Sensor or tree not found"),
        (status = 409, description = "Conflict: sensor not activated or tree already linked"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn set_sensor_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Json(body): Json<SetSensorTreeRequest>,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let sensor_orgs = scope::effective_orgs(sensor.organization_id, &sensor.shared_with);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &sensor_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Sensor, Action::Update),
            &sensor_orgs,
        )
        .await?;

    let tree = state.tree_service.view_by_id(Id::new(body.tree_id)).await?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Tree, Action::Update),
            &scope::effective_orgs(tree.organization_id, &tree.shared_with),
        )
        .await?;

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
        (status = 403, description = "Missing sensor:update or tree:update in the effective organizations"),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn remove_sensor_tree(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
) -> Result<Json<SensorResponse>, ServiceError> {
    let sensor = state.sensor_service.view_by_id(&sensor_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let sensor_orgs = scope::effective_orgs(sensor.organization_id, &sensor.shared_with);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Sensor, Action::Read),
        &sensor_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Sensor, Action::Update),
            &sensor_orgs,
        )
        .await?;

    if let Some(tree) = state.tree_service.view_by_sensor_id(&sensor_id).await? {
        state
            .authorization_service
            .require_any_of(
                user.id,
                Permission::new(Resource::Tree, Action::Update),
                &scope::effective_orgs(tree.organization_id, &tree.shared_with),
            )
            .await?;
    }

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

#[utoipa::path(patch, path = "/sensors/{sensor_id}/organization", tag = "Sensors",
    operation_id = "transferSensor", summary = "Transfer a sensor's ownership to another organization",
    description = "Moves an unbound sensor to a different owning organization. Requires \
                   `sensor:update` in both the source and target organization. A sensor bound \
                   to a tree must be transferred via its tree instead.",
    params(("sensor_id" = String, Path, description = "Sensor ID (EUI)")),
    request_body = TransferRequest,
    responses(
        (status = 204, description = "Sensor transferred"),
        (status = 403, description = "Missing sensor:update in source or target organization"),
        (status = 404, description = "Sensor or organization not found"),
        (status = 409, description = "Sensor is bound to a tree"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(sensor.id = %sensor_id))]
pub async fn transfer_sensor(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    SensorIdPath(sensor_id): SensorIdPath,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = state.sensor_service.view_by_id(&sensor_id).await?;
    let perm = Permission::new(Resource::Sensor, Action::Update);
    state
        .authorization_service
        .require(user.id, perm, Id::new(current.organization_id))
        .await?;
    state
        .authorization_service
        .require(user.id, perm, Id::new(req.organization_id))
        .await?;
    state
        .sensor_service
        .transfer(&sensor_id, Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
