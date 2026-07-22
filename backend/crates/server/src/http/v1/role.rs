use std::collections::BTreeSet;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{AppState, auth::extractor::AuthUserExtractor},
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    role::{RoleDescription, RoleDraft, RoleName},
};

use super::dto::role::{RoleCreateRequest, RoleResponse, RoleUpdateRequest, parse_permissions};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_templates))
        .routes(routes!(list_permissions))
        .routes(routes!(list_org_roles, create_role))
        .routes(routes!(get_role, update_role, delete_role))
}

#[utoipa::path(get, path = "/roles/templates", tag = "Roles",
    operation_id = "listRoleTemplates",
    summary = "List the global role templates",
    description = "Returns the five seeded templates (organization_id = null). Templates are immutable and not assignable.",
    responses(
        (status = 200, description = "The role templates", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
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
        (status = 401, description = "Unauthorized"),
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
    description = "Returns every role instantiated for the given organization, including the copies of the five templates created alongside it.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "Roles owned by the organization", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn list_org_roles(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let views = state.role_service.by_organization(Id::new(org_id)).await?;
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
        (status = 400, description = "Invalid input (missing fields or unknown permission)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Source role/template not found"),
        (status = 409, description = "Name conflict"),
        (status = 500, description = "Internal server error"),
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
                })
                .await?
        }
    };
    Ok((StatusCode::CREATED, Json((&view).into())))
}

#[utoipa::path(get, path = "/roles/{role_id}", tag = "Roles",
    operation_id = "getRole",
    summary = "Get a single role",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 200, description = "The role", body = RoleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(role.id = %role_id))]
pub async fn get_role(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleResponse>, ServiceError> {
    let view = state.role_service.by_id(Id::new(role_id)).await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(patch, path = "/roles/{role_id}", tag = "Roles",
    operation_id = "updateRole",
    summary = "Replace a role's name, description and permissions",
    description = "Templates cannot be modified (409). Requires role:update in the role's organization, plus a permission set that does not exceed the caller's own grants.",
    params(("role_id" = Uuid, Path, description = "Role id")),
    request_body = RoleUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = RoleResponse),
        (status = 400, description = "Invalid input (unknown permission)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Templates are immutable, or name conflicts"),
        (status = 500, description = "Internal server error"),
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
    description = "Templates cannot be deleted (409). Requires role:delete in the role's organization.",
    params(("role_id" = Uuid, Path, description = "Role id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Templates are immutable"),
        (status = 500, description = "Internal server error"),
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
    state.role_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
