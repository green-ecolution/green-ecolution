use std::sync::Arc;

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use waste_domain::settings::FullThreshold;

use super::{WasteResolution, WasteSettingsUpdate};
use crate::http::state::WasteHttpState;
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

pub fn routes() -> OpenApiRouter<Arc<WasteHttpState>> {
    OpenApiRouter::new().routes(routes!(get_waste_settings, update_waste_settings))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WasteSettingsResponse {
    /// Fill level in percent from which a bin counts as full.
    pub full_threshold: SettingField<i64>,
}

/// An omitted field stays as it is. Sending the inherited value drops the
/// organization's own one.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WasteSettingsUpdateRequest {
    #[serde(default)]
    pub full_threshold: Option<i64>,
}

impl TryFrom<WasteSettingsUpdateRequest> for WasteSettingsUpdate {
    type Error = ServiceError;

    fn try_from(req: WasteSettingsUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            full_threshold: patch(req.full_threshold, |v| Ok(FullThreshold::new(v)?))?,
        })
    }
}

impl WasteSettingsResponse {
    fn build(resolution: &WasteResolution) -> Self {
        Self {
            full_threshold: field(
                i64::from(resolution.effective.full_threshold.percent()),
                i64::from(resolution.inherited.full_threshold.percent()),
            ),
        }
    }
}

async fn respond(
    state: &WasteHttpState,
    org: Id<domain::organization::Organization>,
) -> Result<Json<WasteSettingsResponse>, ServiceError> {
    let resolution = state.settings.resolution(org).await?;
    Ok(Json(WasteSettingsResponse::build(&resolution)))
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings/waste", tag = "WasteBins",
    operation_id = "getWasteSettings",
    summary = "Read an organization's waste defaults",
    description = "Each field carries the value in force and the value that would apply without this organization's own one. Requires setting:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The resolved settings", body = WasteSettingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn get_waste_settings(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<WasteSettingsResponse>, ServiceError> {
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

#[utoipa::path(put, path = "/organizations/{org_id}/settings/waste", tag = "WasteBins",
    operation_id = "updateWasteSettings",
    summary = "Set an organization's waste defaults",
    description = "An omitted field stays as it is. Sending the value the organization would inherit drops its own value, so it follows its parent again. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = WasteSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = WasteSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn update_waste_settings(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<WasteSettingsUpdateRequest>,
) -> Result<Json<WasteSettingsResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .access
        .require_write(
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;

    let update = WasteSettingsUpdate::try_from(req)?;
    state.settings.update(user.id, org, update).await?;
    respond(&state, org).await
}
