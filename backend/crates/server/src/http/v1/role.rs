use std::collections::BTreeSet;

use crate::http::v1::error::ErrorBody;
use std::sync::Arc;

use crate::http::extractors::{Json, Path};
use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::scope::{ensure_visible, resolve_target_org},
    },
    service::{ServiceError, authorization::RoleChange},
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource, Visibility},
    organization::Organization,
    role::{RoleDescription, RoleDraft, RoleName},
};

use super::dto::role::{RoleCreateRequest, RoleResponse, RoleUpdateRequest, parse_permissions};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_roles))
        .routes(routes!(list_templates))
        .routes(routes!(list_permissions))
        .routes(routes!(list_org_roles, create_role))
        .routes(routes!(get_role, update_role, delete_role))
}

#[utoipa::path(get, path = "/roles", tag = "Roles",
    operation_id = "listRoles",
    summary = "List roles visible to the caller",
    description = "Returns every non-template role owned by an organization in the caller's visible subtree. Templates (organization_id = null) are excluded, since they cannot be assigned. Requires role:read.",
    responses(
        (status = 200, description = "Roles visible to the caller", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 422, description = "Acting user has no organization and none was given", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let read = Permission::new(Resource::Role, Action::Read);
    let scope = resolve_target_org(&state, user.id, None).await?;
    state
        .authorization_service
        .require(user.id, read, scope)
        .await?;
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, read)
        .await?;
    let org_ids: Vec<Id<Organization>> = match visible {
        // Demo bypass: no orgs are actually excluded, so every organization
        // stands in for "the caller's visible subtree".
        Visibility::Unrestricted => state
            .organization_service
            .list()
            .await?
            .iter()
            .map(|org| org.id)
            .collect(),
        Visibility::Only(orgs) => orgs.into_iter().collect(),
    };
    let views = state.role_service.by_organizations(&org_ids).await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/roles/templates", tag = "Roles",
    operation_id = "listRoleTemplates",
    summary = "List the global role templates",
    description = "Returns the five seeded templates (organization_id = null). Templates are immutable and not assignable.",
    responses(
        (status = 200, description = "The role templates", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let views = state.role_service.templates().await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/permissions", tag = "Roles",
    operation_id = "listPermissions",
    summary = "List the permission catalog",
    description = "Returns every grantable permission as `<resource>:<action>`, e.g. `tree:read`.",
    responses(
        (status = 200, description = "The permission catalog", body = Vec<String>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_permissions(_user: AuthUserExtractor) -> Json<Vec<String>> {
    Json(
        Permission::catalog()
            .iter()
            .map(ToString::to_string)
            .collect(),
    )
}

#[utoipa::path(get, path = "/organizations/{org_id}/roles", tag = "Roles",
    operation_id = "listOrgRoles",
    summary = "List roles owned by an organization",
    description = "Returns every role instantiated for the given organization, including the copies of the five templates created alongside it. Requires role:read in that organization.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "Roles owned by the organization", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn list_org_roles(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let id = Id::new(org_id);
    state
        .authorization_service
        .require(user.id, Permission::new(Resource::Role, Action::Read), id)
        .await?;
    let views = state.role_service.by_organization(id).await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(post, path = "/organizations/{org_id}/roles", tag = "Roles",
    operation_id = "createRole",
    summary = "Create a role, from scratch or as a copy",
    description = "Either provide `name` + `permissions` to create a role from scratch, or `copy_from_role_id` to copy an existing role/template into this organization (optionally renaming it). Requires role:create in the organization, plus a permission set that does not exceed the caller's own grants.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = RoleCreateRequest,
    responses(
        (status = 201, description = "Created", body = RoleResponse),
        (status = 400, description = "Invalid input (missing fields or unknown permission)", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Source role/template not found", body = ErrorBody),
        (status = 409, description = "Name conflict (code `resource.already_exists`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<RoleCreateRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), ServiceError> {
    let org = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Role, Action::Create),
            org,
        )
        .await?;
    let view = match req.copy_from_role_id {
        Some(source) => {
            let source_view = state.role_service.by_id(Id::new(source)).await?;
            let perms: BTreeSet<Permission> = source_view.permissions.iter().copied().collect();
            state
                .authorization_service
                .require_superset(user.id, &perms, org)
                .await?;
            match req.name {
                Some(name) => {
                    state
                        .role_service
                        .create(RoleDraft {
                            organization_id: org,
                            name: RoleName::new(name)?,
                            description: source_view.description.clone(),
                            permissions: perms,
                            // A renamed copy is the caller's own role, not the
                            // delivered one, so it carries no template key.
                            template_key: None,
                        })
                        .await?
                }
                None => state.role_service.copy(Id::new(source), org).await?,
            }
        }
        None => {
            let name = req.name.ok_or_else(|| {
                ServiceError::InvalidInput(
                    "name is required unless copy_from_role_id is set".into(),
                )
            })?;
            let permissions = parse_permissions(&req.permissions.unwrap_or_default())?;
            state
                .authorization_service
                .require_superset(user.id, &permissions, org)
                .await?;
            let description = req
                .description
                .filter(|s| !s.is_empty())
                .map(RoleDescription::new)
                .transpose()?;
            state
                .role_service
                .create(RoleDraft {
                    organization_id: org,
                    name: RoleName::new(name)?,
                    description,
                    permissions,
                    template_key: None,
                })
                .await?
        }
    };
    Ok((StatusCode::CREATED, Json((&view).into())))
}

#[utoipa::path(get, path = "/roles/{role_id}", tag = "Roles",
    operation_id = "getRole",
    summary = "Get a single role",
    description = "Requires role:read in the role's organization; a role outside the caller's visible subtree answers 404 rather than 403 so callers cannot probe for existence. Templates carry no organization and stay readable.",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 200, description = "The role", body = RoleResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Not found, or not visible to the caller", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(role.id = %role_id))]
pub async fn get_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleResponse>, ServiceError> {
    let view = state.role_service.by_id(Id::new(role_id)).await?;
    // Templates own no organization and are already listed unscoped under
    // /roles/templates; scoping them would only break the copy-on-create flow.
    if let Some(org) = view.organization_id {
        let ctx = state.authorization_service.context_for(user.id).await?;
        ensure_visible(
            &ctx,
            Permission::new(Resource::Role, Action::Read),
            org.value(),
        )?;
    }
    Ok(Json((&view).into()))
}

#[utoipa::path(patch, path = "/roles/{role_id}", tag = "Roles",
    operation_id = "updateRole",
    summary = "Replace a role's name, description and permissions",
    description = "Templates cannot be modified (409). Requires role:update in the role's organization, plus a permission set that does not exceed the caller's own grants. A change that would leave the caller without role:update or user:update in their own organization is rejected (409).",
    params(("role_id" = Uuid, Path, description = "Role id")),
    request_body = RoleUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = RoleResponse),
        (status = 400, description = "Invalid input (unknown permission)", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "Templates are immutable, the name conflicts, or the change would revoke the caller's own administration rights (codes `conflict.role_template_immutable`, `resource.already_exists`, `conflict.cannot_revoke_own_administration`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(role.id = %role_id))]
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(role_id): Path<Uuid>,
    Json(req): Json<RoleUpdateRequest>,
) -> Result<Json<RoleResponse>, ServiceError> {
    let id = Id::new(role_id);
    let current = state.role_service.by_id(id).await?;
    let permissions = parse_permissions(&req.permissions)?;
    if let Some(org) = current.organization_id {
        state
            .authorization_service
            .require(
                user.id,
                Permission::new(Resource::Role, Action::Update),
                org,
            )
            .await?;
        state
            .authorization_service
            .require_superset(user.id, &permissions, org)
            .await?;
    }
    state
        .authorization_service
        .ensure_administration_survives(
            user.id,
            state.user_service.organization_of(user.id).await?,
            id,
            RoleChange::PermissionsReplacedWith(&permissions),
        )
        .await?;
    let description = req
        .description
        .filter(|s| !s.is_empty())
        .map(RoleDescription::new)
        .transpose()?;
    let updated = state
        .role_service
        .update(id, RoleName::new(req.name)?, description, permissions)
        .await?;
    Ok(Json((&updated).into()))
}

#[utoipa::path(delete, path = "/roles/{role_id}", tag = "Roles",
    operation_id = "deleteRole",
    summary = "Delete a role",
    description = "Templates cannot be deleted (409). Requires role:delete in the role's organization. Deleting a role that carries the caller's own role:update or user:update in their organization is rejected (409).",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 409, description = "Templates are immutable, or the deletion would revoke the caller's own administration rights (codes `conflict.role_template_immutable`, `conflict.cannot_revoke_own_administration`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(role.id = %role_id))]
pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, ServiceError> {
    let id = Id::new(role_id);
    let current = state.role_service.by_id(id).await?;
    if let Some(org) = current.organization_id {
        state
            .authorization_service
            .require(
                user.id,
                Permission::new(Resource::Role, Action::Delete),
                org,
            )
            .await?;
    }
    state
        .authorization_service
        .ensure_administration_survives(
            user.id,
            state.user_service.organization_of(user.id).await?,
            id,
            RoleChange::Deleted,
        )
        .await?;
    state.role_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
