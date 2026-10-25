use std::{collections::HashMap, sync::Arc};

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
    organization::OrganizationReader,
};
use irrigation_domain::settings::{JustWateredTtl, WaterDemand};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    IrrigationResolution, IrrigationSettings, IrrigationSettingsUpdate,
    service::IrrigationSettingsService,
};
use crate::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{
            dto::settings::{
                ChangeStamp, OrganizationRef, SettingField, double_option, field, patch,
                resolve_change_authors,
            },
            error::ErrorBody,
        },
    },
    service::{
        ServiceError, authorization::AuthorizationService, module_access::ModuleAccess,
        user_service::UserService,
    },
};

pub struct IrrigationHttpState {
    pub settings: Arc<IrrigationSettingsService>,
    pub access: ModuleAccess,
    pub authorization: Arc<AuthorizationService>,
    pub organizations: Arc<dyn OrganizationReader>,
    pub users: Arc<UserService>,
}

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new().routes(routes!(get_irrigation_settings, update_irrigation_settings))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IrrigationSettingsResponse {
    /// Liters per tree.
    pub water_demand: SettingField<f64>,
    pub just_watered_ttl_secs: SettingField<i64>,
    pub enforced_by: Option<OrganizationRef>,
}

/// Three states per field. Absent means unchanged, `null` means go back to
/// inheriting, a number sets it.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IrrigationSettingsUpdateRequest {
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub water_demand: Option<Option<f64>>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<i64>)]
    pub just_watered_ttl_secs: Option<Option<i64>>,
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
    fn build(
        resolution: &IrrigationResolution,
        own: &IrrigationSettings,
        last: &HashMap<&'static str, ChangeStamp>,
        names: &HashMap<Uuid, String>,
    ) -> Self {
        let effective = resolution.effective;
        let origins = resolution.origins;
        Self {
            water_demand: field(
                effective.water_demand.liters(),
                origins.water_demand,
                own.water_demand.map(|v| v.liters()),
                last.get("water_demand").copied(),
                names,
            ),
            just_watered_ttl_secs: field(
                effective.just_watered_ttl.seconds(),
                origins.just_watered_ttl,
                own.just_watered_ttl.map(|v| v.seconds()),
                last.get("just_watered_ttl").copied(),
                names,
            ),
            enforced_by: effective
                .enforced_by
                .map(|id| OrganizationRef { id: id.value() }),
        }
    }
}

async fn respond(
    state: &IrrigationHttpState,
    org: Id<domain::organization::Organization>,
) -> Result<Json<IrrigationSettingsResponse>, ServiceError> {
    let resolution = state.settings.resolution(org).await?;
    let own = state.settings.own(org).await?;
    let last = state.settings.last_changes(org).await?;
    let names = resolve_change_authors(&state.users, last.values().map(|e| e.changed_by)).await;
    Ok(Json(IrrigationSettingsResponse::build(
        &resolution,
        &own,
        &last,
        &names,
    )))
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings/irrigation", tag = "Irrigation",
    operation_id = "getIrrigationSettings",
    summary = "Read an organization's irrigation defaults",
    description = "Each field carries the value in force, where it came from, this organization's own value and when it was last changed. Requires setting:read.",
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
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;
    respond(&state, org).await
}

#[utoipa::path(put, path = "/organizations/{org_id}/settings/irrigation", tag = "Irrigation",
    operation_id = "updateIrrigationSettings",
    summary = "Set an organization's irrigation defaults",
    description = "An omitted field stays as it is, `null` drops the organization's own value and returns it to inheritance. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = IrrigationSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = IrrigationSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "An organization above froze this subtree (code `settings.enforced_by_ancestor`)", body = ErrorBody),
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
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;

    let update = IrrigationSettingsUpdate::try_from(req)?;
    state.settings.update(user.id, org, update).await?;
    respond(&state, org).await
}
