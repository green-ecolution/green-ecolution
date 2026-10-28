use std::sync::Arc;

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::dto::module::{
            ModuleListResponse, OrganizationModulesEntry, OrganizationModulesResponse,
            OrganizationModulesUpdateRequest,
        },
        v1::error::ErrorBody,
    },
    service::{ServiceError, module_activation_service::ModuleActivationService},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_modules))
        .routes(routes!(
            get_organization_modules,
            update_organization_modules
        ))
}

#[utoipa::path(get, path = "/modules", tag = "Modules",
    operation_id = "listModules",
    summary = "List modules and where they are active",
    description = "Every module the instance ships plus, for each organization the caller works in, the modules active there.",
    responses(
        (status = 200, description = "Modules", body = ModuleListResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_modules(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<ModuleListResponse>, ServiceError> {
    let reachable = state
        .authorization_service
        .context_for(user.id)
        .await?
        .reachable_orgs();
    let active = state
        .module_activation_service
        .active_by_org(&reachable)
        .await?;
    Ok(Json(ModuleListResponse {
        modules: state
            .module_activation_service
            .registered()
            .iter()
            .map(ToString::to_string)
            .collect(),
        organizations: active
            .into_iter()
            .map(|(org, keys)| OrganizationModulesEntry {
                organization_id: org.value(),
                active: keys.iter().map(ToString::to_string).collect(),
            })
            .collect(),
    }))
}

#[utoipa::path(get, path = "/organizations/{org_id}/modules", tag = "Modules",
    operation_id = "getOrganizationModules",
    summary = "Read an organization's module switches",
    description = "Each module carries the state in force, where it came from and this organization's own switch. Requires setting:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The resolved switches", body = OrganizationModulesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn get_organization_modules(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationModulesResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;
    respond(&state, user.id, org).await
}

async fn respond(
    state: &AppState,
    user_id: Uuid,
    org: Id<domain::organization::Organization>,
) -> Result<Json<OrganizationModulesResponse>, ServiceError> {
    let ctx = state.authorization_service.context_for(user_id).await?;
    let resolution = state.module_activation_service.resolution(org).await?;
    let own = state.module_activation_service.own(org).await?;
    Ok(Json(OrganizationModulesResponse::build(
        &resolution,
        &own,
        |switch| ModuleActivationService::may_change(&ctx, switch),
    )))
}

#[utoipa::path(put, path = "/organizations/{org_id}/modules", tag = "Modules",
    operation_id = "updateOrganizationModules",
    summary = "Switch modules for an organization",
    description = "An omitted module stays as it is, `null` returns it to inheritance. A switch is stamped with the topmost organization the caller may update settings for; only callers acting for that organization or above may change it again. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationModulesUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = OrganizationModulesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "An organization above froze this subtree (code `settings.enforced_by_ancestor`), or set a switch for this organization that the caller does not act for (code `settings.set_by_ancestor`)", body = ErrorBody),
        (status = 422, description = "Unknown module (code `module.unknown`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn update_organization_modules(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<OrganizationModulesUpdateRequest>,
) -> Result<Json<OrganizationModulesResponse>, ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    state
        .module_activation_service
        .update(&ctx, org, req.modules)
        .await?;
    respond(&state, user.id, org).await
}
