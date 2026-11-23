use std::sync::Arc;

use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::dto::settings::{OrganizationSettingsResponse, OrganizationSettingsUpdateRequest},
        v1::error::ErrorBody,
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
    settings::SettingsUpdate,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_settings, update_settings))
}

#[utoipa::path(get, path = "/organizations/{org_id}/settings", tag = "Settings",
    operation_id = "getOrganizationSettings",
    summary = "Read an organization's operational defaults",
    description = "Each field carries the value in force and the value that would apply without this organization's own one. Requires setting:read.",
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
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;

    let resolution = state.settings_service.resolution(org).await?;
    Ok(Json(OrganizationSettingsResponse::build(&resolution)))
}

#[utoipa::path(put, path = "/organizations/{org_id}/settings", tag = "Settings",
    operation_id = "updateOrganizationSettings",
    summary = "Set an organization's operational defaults",
    description = "An omitted field stays as it is. Sending the value the organization would inherit drops its own value, so it follows its parent again. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationSettingsUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = OrganizationSettingsResponse),
        (status = 400, description = "Value outside its range", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
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
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;

    let update = SettingsUpdate::try_from(req)?;
    state.settings_service.update(user.id, org, update).await?;

    let resolution = state.settings_service.resolution(org).await?;
    Ok(Json(OrganizationSettingsResponse::build(&resolution)))
}
