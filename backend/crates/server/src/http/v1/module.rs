use std::sync::Arc;

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Permission, ResourceKey},
    organization::root_organization_id,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::dto::module::{
            ModuleListResponse, ModuleReleasesResponse, ModuleReleasesUpdateRequest,
            OrganizationModulesEntry, OrganizationModulesResponse,
            OrganizationModulesUpdateRequest, TenantModuleReleasesResponse,
        },
        v1::error::ErrorBody,
    },
    service::{AuthError, ServiceError},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_modules))
        .routes(routes!(list_module_releases))
        .routes(routes!(update_organization_module_releases))
        .routes(routes!(
            get_organization_modules,
            update_organization_modules
        ))
}

#[utoipa::path(get, path = "/modules", tag = "Modules",
    operation_id = "listModules",
    summary = "List released modules and where they are active",
    description = "Every module released in an organization the caller works in plus, for each organization the caller works in, the modules active there.",
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
        .context_for(user.caller())
        .await?
        .reachable_orgs();
    let listing = state.module_activation_service.listing(&reachable).await?;
    Ok(Json(ModuleListResponse {
        modules: listing.released.iter().map(ToString::to_string).collect(),
        organizations: listing
            .active
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
    description = "Lists the modules released for the organization's tenant with their switch. A locked module is absent. Requires setting:read.",
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
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Read),
            org,
        )
        .await?;
    respond(&state, org).await
}

async fn respond(
    state: &AppState,
    org: Id<domain::organization::Organization>,
) -> Result<Json<OrganizationModulesResponse>, ServiceError> {
    let states = state.module_activation_service.states(org).await?;
    let switchable = state.module_activation_service.switchable(org).await?;
    Ok(Json(OrganizationModulesResponse::build(
        &states, switchable,
    )))
}

#[utoipa::path(put, path = "/organizations/{org_id}/modules", tag = "Modules",
    operation_id = "updateOrganizationModules",
    summary = "Switch modules for an organization",
    description = "Switches released modules on or off. Only a tenant and the root organization hold switches; a sub-organization answers 422 `module.not_switchable`. A locked module answers like an unknown one. Requires setting:update.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationModulesUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = OrganizationModulesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 422, description = "Unknown or locked module (code `module.unknown`) or an organization that holds no switches (code `module.not_switchable`)", body = ErrorBody),
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
            user.caller(),
            Permission::new(ResourceKey::SETTING, Action::Update),
            org,
        )
        .await?;
    state
        .module_activation_service
        .update(org, req.modules)
        .await?;
    respond(&state, org).await
}

/// Releasing belongs to the root organization: only its members can hold a
/// right there. Judged without the scope, so a root member working inside a
/// tenant still acts for the root.
async fn require_at_root(
    state: &AppState,
    caller: domain::authorization::Caller,
    action: Action,
) -> Result<(), ServiceError> {
    let ctx = state
        .authorization_service
        .context_for(caller)
        .await?
        .without_scope();
    if ctx.allows_in(
        Permission::new(ResourceKey::SETTING, action),
        root_organization_id(),
    ) {
        Ok(())
    } else {
        Err(AuthError::Forbidden.into())
    }
}

#[utoipa::path(get, path = "/modules/releases", tag = "Modules",
    operation_id = "listModuleReleases",
    summary = "List module releases per tenant",
    description = "Every tenant with the release state and the own switch of each module this instance ships. Requires setting:read at the root organization.",
    responses(
        (status = 200, description = "Releases per tenant", body = ModuleReleasesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_module_releases(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<ModuleReleasesResponse>, ServiceError> {
    require_at_root(&state, user.caller(), Action::Read).await?;
    let tenants = state.module_activation_service.releases().await?;
    Ok(Json(ModuleReleasesResponse {
        modules: state
            .module_activation_service
            .registered()
            .iter()
            .map(ToString::to_string)
            .collect(),
        tenants: tenants
            .iter()
            .map(|(tenant, states)| TenantModuleReleasesResponse::build(tenant.value(), states))
            .collect(),
    }))
}

#[utoipa::path(put, path = "/organizations/{org_id}/module-releases", tag = "Modules",
    operation_id = "updateOrganizationModuleReleases",
    summary = "Release or lock modules for a tenant",
    description = "Releases or locks modules for a tenant. The first release switches the module on; locking keeps the tenant's switch. Requires setting:update at the root organization.",
    params(("org_id" = Uuid, Path, description = "Tenant id")),
    request_body = ModuleReleasesUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = TenantModuleReleasesResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 422, description = "Unknown module (code `module.unknown`) or a target that is no tenant (code `module.not_a_tenant`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn update_organization_module_releases(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<ModuleReleasesUpdateRequest>,
) -> Result<Json<TenantModuleReleasesResponse>, ServiceError> {
    require_at_root(&state, user.caller(), Action::Update).await?;
    let tenant = Id::new(org_id);
    let states = state
        .module_activation_service
        .release(tenant, req.modules)
        .await?;
    Ok(Json(TenantModuleReleasesResponse::build(org_id, &states)))
}
