use std::{collections::HashMap, sync::Arc};

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use waste_domain::settings::FullThreshold;

use super::{WasteResolution, WasteSettings, WasteSettingsUpdate, repository::FULL_THRESHOLD_KEY};
use crate::http::state::WasteHttpState;
use server::{
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
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<WasteHttpState>> {
    OpenApiRouter::new().routes(routes!(get_waste_settings, update_waste_settings))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WasteSettingsResponse {
    /// Fill level in percent from which a bin counts as full.
    pub full_threshold: SettingField<i64>,
    pub enforced_by: Option<OrganizationRef>,
}

/// Three states per field. Absent means unchanged, `null` means go back to
/// inheriting, a number sets it.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WasteSettingsUpdateRequest {
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<i64>)]
    pub full_threshold: Option<Option<i64>>,
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
    fn build(
        resolution: &WasteResolution,
        own: &WasteSettings,
        last: &HashMap<&'static str, ChangeStamp>,
        names: &HashMap<Uuid, String>,
    ) -> Self {
        let effective = resolution.effective;
        Self {
            full_threshold: field(
                i64::from(effective.full_threshold.percent()),
                resolution.origins.full_threshold,
                own.full_threshold.map(|v| i64::from(v.percent())),
                last.get(FULL_THRESHOLD_KEY).copied(),
                names,
            ),
            enforced_by: effective
                .enforced_by
                .map(|id| OrganizationRef { id: id.value() }),
        }
    }
}

async fn respond(
    state: &WasteHttpState,
    org: Id<domain::organization::Organization>,
) -> Result<Json<WasteSettingsResponse>, ServiceError> {
    let resolution = state.settings.resolution(org).await?;
    let own = state.settings.own(org).await?;
    let last = state.settings.last_changes(org).await?;
    let names = resolve_change_authors(&state.users, last.values().map(|e| e.changed_by)).await;
    Ok(Json(WasteSettingsResponse::build(
        &resolution,
        &own,
        &last,
        &names,
    )))
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings/waste", tag = "WasteBins",
    operation_id = "getWasteSettings",
    summary = "Read an organization's waste defaults",
    description = "Each field carries the value in force, where it came from, this organization's own value and when it was last changed. Requires setting:read.",
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
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;
    respond(&state, org).await
}

#[utoipa::path(put, path = "/organizations/{org_id}/settings/waste", tag = "WasteBins",
    operation_id = "updateWasteSettings",
    summary = "Set an organization's waste defaults",
    description = "An omitted field stays as it is, `null` drops the organization's own value and returns it to inheritance. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = WasteSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = WasteSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "An organization above froze this subtree (code `settings.enforced_by_ancestor`)", body = ErrorBody),
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
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;

    let update = WasteSettingsUpdate::try_from(req)?;
    state.settings.update(user.id, org, update).await?;
    respond(&state, org).await
}
