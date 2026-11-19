use std::sync::Arc;

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
};
use irrigation_domain::settings::{JustWateredTtl, WaterDemand};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{IrrigationResolution, IrrigationSettingsUpdate};
use crate::http::state::IrrigationHttpState;
use server::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{
            dto::settings::{SettingField, field, patch},
            error::ErrorBody,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new().routes(routes!(get_irrigation_settings, update_irrigation_settings))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IrrigationSettingsResponse {
    /// Liters per tree.
    pub water_demand: SettingField<f64>,
    pub just_watered_ttl_secs: SettingField<i64>,
}

/// An omitted field stays as it is. Sending the inherited value drops the
/// organization's own one.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IrrigationSettingsUpdateRequest {
    #[serde(default)]
    pub water_demand: Option<f64>,
    #[serde(default)]
    pub just_watered_ttl_secs: Option<i64>,
}

impl TryFrom<IrrigationSettingsUpdateRequest> for IrrigationSettingsUpdate {
    type Error = ServiceError;

    fn try_from(req: IrrigationSettingsUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            water_demand: patch(req.water_demand, |v| Ok(WaterDemand::new(v)?))?,
            just_watered_ttl: patch(req.just_watered_ttl_secs, |v| Ok(JustWateredTtl::new(v)?))?,
        })
    }
}

impl IrrigationSettingsResponse {
    fn build(resolution: &IrrigationResolution) -> Self {
        let IrrigationResolution {
            effective,
            inherited,
        } = *resolution;
        Self {
            water_demand: field(
                effective.water_demand.liters(),
                inherited.water_demand.liters(),
            ),
            just_watered_ttl_secs: field(
                effective.just_watered_ttl.seconds(),
                inherited.just_watered_ttl.seconds(),
            ),
        }
    }
}

async fn respond(
    state: &IrrigationHttpState,
    org: Id<domain::organization::Organization>,
) -> Result<Json<IrrigationSettingsResponse>, ServiceError> {
    let resolution = state.settings.resolution(org).await?;
    Ok(Json(IrrigationSettingsResponse::build(&resolution)))
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings/irrigation", tag = "Irrigation",
    operation_id = "getIrrigationSettings",
    summary = "Read an organization's irrigation defaults",
    description = "Each field carries the value in force and the value that would apply without this organization's own one. Requires setting:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The resolved settings", body = IrrigationSettingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn get_irrigation_settings(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<IrrigationSettingsResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization
        .require(
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;
    respond(&state, org).await
}

#[utoipa::path(put, path = "/organizations/{org_id}/settings/irrigation", tag = "Irrigation",
    operation_id = "updateIrrigationSettings",
    summary = "Set an organization's irrigation defaults",
    description = "An omitted field stays as it is. Sending the value the organization would inherit drops its own value, so it follows its parent again. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = IrrigationSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = IrrigationSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn update_irrigation_settings(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<IrrigationSettingsUpdateRequest>,
) -> Result<Json<IrrigationSettingsResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .access
        .require_write(
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;

    let update = IrrigationSettingsUpdate::try_from(req)?;
    state.settings.update(user.id, org, update).await?;
    respond(&state, org).await
}
