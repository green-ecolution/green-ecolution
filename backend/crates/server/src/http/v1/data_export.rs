use std::sync::Arc;

use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{
            dto::data_export::{
                ConnectionTestResponse, DataExportTargetRequest, DataExportTargetResponse,
            },
            error::ErrorBody,
        },
    },
    service::{Feature, ServiceError},
};
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
    data_export::ExportTargetDraft,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(
            get_data_export,
            put_data_export,
            delete_data_export
        ))
        .routes(routes!(test_data_export))
}

async fn guard(
    state: &AppState,
    user: &AuthUserExtractor,
    org: Id<domain::organization::Organization>,
    action: Action,
) -> Result<(), ServiceError> {
    if !state.feature_flags.data_export_enabled {
        return Err(ServiceError::FeatureDisabled {
            feature: Feature::DataExport,
        });
    }
    state
        .authorization_service
        .require(
            user.caller(),
            Permission::new(ResourceKey::SETTING, action),
            org,
        )
        .await?;
    Ok(())
}

#[utoipa::path(get, path = "/organizations/{org_id}/data-export", tag = "Data Export",
    operation_id = "getDataExportTarget",
    summary = "Read an organization's data platform export target",
    description = "The target (a SensorThings server) serves the organization's subtree. The secret is never returned, only `has_secret`. Requires setting:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The export target", body = DataExportTargetResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "No target configured", body = ErrorBody),
        (status = 503, description = "Data export is disabled on this instance (code `feature.data_export_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn get_data_export(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<DataExportTargetResponse>, ServiceError> {
    let org = Id::new(org_id);
    guard(&state, &user, org, Action::Read).await?;
    Ok(Json(state.data_export_service.get(org).await?.into()))
}

#[utoipa::path(put, path = "/organizations/{org_id}/data-export", tag = "Data Export",
    operation_id = "putDataExportTarget",
    summary = "Create or replace an organization's export target",
    description = "Omitting `password` / `client_secret` keeps the stored secret of the same auth kind. A changed `base_url` counts as a new target. Hosts must be on the instance allowlist. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = DataExportTargetRequest,
    responses(
        (status = 200, description = "Stored", body = DataExportTargetResponse),
        (status = 400, description = "Invalid input (code `request.validation_failed`)", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Organization not found", body = ErrorBody),
        (status = 422, description = "Host not allowed (code `data_export.url_not_allowed`) or secret missing (code `data_export.auth_incomplete`)", body = ErrorBody),
        (status = 503, description = "Data export is disabled on this instance (code `feature.data_export_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn put_data_export(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<DataExportTargetRequest>,
) -> Result<Json<DataExportTargetResponse>, ServiceError> {
    let org = Id::new(org_id);
    guard(&state, &user, org, Action::Update).await?;
    let draft = ExportTargetDraft::try_from(req)?;
    Ok(Json(
        state.data_export_service.put(org, draft).await?.into(),
    ))
}

#[utoipa::path(delete, path = "/organizations/{org_id}/data-export", tag = "Data Export",
    operation_id = "deleteDataExportTarget",
    summary = "Remove an organization's export target",
    description = "Stops the export. Nothing is deleted on the data platform. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 204, description = "Removed"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "No target configured", body = ErrorBody),
        (status = 503, description = "Data export is disabled on this instance (code `feature.data_export_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn delete_data_export(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<StatusCode, ServiceError> {
    let org = Id::new(org_id);
    guard(&state, &user, org, Action::Update).await?;
    state.data_export_service.delete(org).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/organizations/{org_id}/data-export/test", tag = "Data Export",
    operation_id = "testDataExportTarget",
    summary = "Test the connection to the stored export target",
    description = "Fetches the SensorThings service root with the stored auth. A failed connection is a normal 200 answer whose `outcome` says why. Requires setting:update, because it sends the stored credentials.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "Test result", body = ConnectionTestResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "No target configured", body = ErrorBody),
        (status = 503, description = "Data export is disabled on this instance (code `feature.data_export_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn test_data_export(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ConnectionTestResponse>, ServiceError> {
    let org = Id::new(org_id);
    guard(&state, &user, org, Action::Update).await?;
    Ok(Json(state.data_export_service.test(org).await?.into()))
}
