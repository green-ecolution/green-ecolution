use std::{collections::HashMap, sync::Arc};

use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::dto::{
            settings::{OrganizationSettingsResponse, OrganizationSettingsUpdateRequest},
            user::display_name,
        },
        v1::error::ErrorBody,
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    settings::{SettingChangeEntry, SettingKey, SettingsUpdate},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_settings, update_settings))
}

/// One lookup for every distinct author on the page. An IdP outage must not
/// fail the read — the values are still valid, they just render without a name.
async fn resolve_change_authors(
    state: &AppState,
    last: &HashMap<SettingKey, SettingChangeEntry>,
) -> HashMap<Uuid, String> {
    let mut ids: Vec<Uuid> = last.values().filter_map(|e| e.changed_by).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return HashMap::new();
    }
    state
        .user_service
        .by_ids(&ids)
        .await
        .inspect_err(|error| tracing::warn!(%error, "failed to resolve settings change authors"))
        .unwrap_or_default()
        .iter()
        .map(|user| (user.id, display_name(user)))
        .collect()
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings", tag = "Settings",
    operation_id = "getOrganizationSettings",
    summary = "Read an organization's operational defaults",
    description = "Each field carries the value in force, where it came from, this organization's own value (even while a lock above makes it dormant) and when it was last changed. Requires setting:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The resolved settings", body = OrganizationSettingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationSettingsResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Setting, Action::Read),
            org,
        )
        .await?;

    let resolution = state.settings_service.resolution(org).await?;
    let own = state.settings_service.own(org).await?;
    let last = state.settings_service.last_changes(org).await?;
    let names = resolve_change_authors(&state, &last).await;
    Ok(Json(OrganizationSettingsResponse::build(
        &resolution,
        &own,
        &last,
        &names,
    )))
}

#[utoipa::path(put, path = "/organizations/{org_id}/settings", tag = "Settings",
    operation_id = "updateOrganizationSettings",
    summary = "Set an organization's operational defaults",
    description = "An omitted field stays as it is, `null` drops the organization's own value and returns it to inheritance. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = OrganizationSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "An organization above froze this subtree (code `settings.enforced_by_ancestor`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<OrganizationSettingsUpdateRequest>,
) -> Result<Json<OrganizationSettingsResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Setting, Action::Update),
            org,
        )
        .await?;

    let update = SettingsUpdate::try_from(req)?;
    state.settings_service.update(user.id, org, update).await?;

    let resolution = state.settings_service.resolution(org).await?;
    let own = state.settings_service.own(org).await?;
    let last = state.settings_service.last_changes(org).await?;
    let names = resolve_change_authors(&state, &last).await;
    Ok(Json(OrganizationSettingsResponse::build(
        &resolution,
        &own,
        &last,
        &names,
    )))
}
