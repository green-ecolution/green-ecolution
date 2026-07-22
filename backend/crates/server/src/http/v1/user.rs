use std::{collections::BTreeSet, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                role::RoleResponse,
                user::{
                    AssignRoleRequest, SetOrganizationRequest, UserListParams, UserRegisterRequest,
                    UserResponse, UserUpdateRequest,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::{AuthError, ServiceError, user_service::UserListFilter},
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    shared::pagination::Pagination,
    user::UserCreate as DomainUserCreate,
};

pub fn protected_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_users, create_user))
        .routes(routes!(get_me))
        .routes(routes!(update_user))
        .routes(routes!(list_user_roles, assign_user_role))
        .routes(routes!(revoke_user_role))
        .routes(routes!(set_user_organization))
}

#[utoipa::path(get, path = "/users/me", tag = "Users",
    operation_id = "getMe",
    summary = "Get the authenticated user",
    description = "Returns the profile of the currently authenticated user.",
    responses(
        (status = 200, description = "The authenticated user", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<UserResponse>, ServiceError> {
    let view = state
        .user_service
        .by_ids(&[user.id])
        .await?
        .into_iter()
        .next()
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    Ok(Json((&view).into()))
}

#[utoipa::path(get, path = "/users", tag = "Users",
    operation_id = "listUsers",
    summary = "List all users",
    description = "Returns a paginated list of registered users, optionally filtered by organization or role.",
    params(
        PaginationParams,
        ("organization_id" = Option<Uuid>, Query, description = "Filter by organization membership"),
        ("role_id" = Option<Uuid>, Query, description = "Filter by assigned role"),
    ),
    responses(
        (status = 200, description = "Paginated list of users", body = ListResponse<UserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Query(params): Query<UserListParams>,
) -> Result<Json<ListResponse<UserResponse>>, ServiceError> {
    let pagination = Pagination::from(&params.pagination);
    let filter = UserListFilter {
        organization_id: params.organization_id.map(Id::new),
        role_id: params.role_id.map(Id::new),
    };
    let page = state.user_service.list(pagination, filter).await?;
    Ok(Json(ListResponse::from_page(page, &pagination)))
}

#[utoipa::path(post, path = "/users", tag = "Users",
    operation_id = "createUser",
    summary = "Register a new user",
    description = "Registers a user in the given organization and assigns the requested roles. Requires user:create in the organization, plus each role's permissions must not exceed the caller's own grants.",
    request_body = UserRegisterRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 409, description = "Role is a template and cannot be assigned"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<UserRegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ServiceError> {
    let entity: DomainUserCreate = req.try_into()?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::User, Action::Create),
            entity.organization_id,
        )
        .await?;
    for role_id in &entity.role_ids {
        let role = state.role_service.by_id(*role_id).await?;
        if let Some(role_org) = role.organization_id {
            let perms: BTreeSet<Permission> = role.permissions.iter().copied().collect();
            state
                .authorization_service
                .require_superset(user.id, &perms, role_org)
                .await?;
        }
    }
    let created = state.user_service.register(entity).await?;
    Ok((StatusCode::CREATED, Json((&created).into())))
}

#[utoipa::path(put, path = "/users/{user_id}", tag = "Users",
    operation_id = "updateUser",
    summary = "Replace a user's profile data",
    description = "Replace-style update of the app-owned user profile. Requires user:update in the target user's organization.",
    params(("user_id" = Uuid, Path, description = "User id")),
    request_body = UserUpdateRequest,
    responses(
        (status = 200, description = "Updated user", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UserUpdateRequest>,
) -> Result<Json<UserResponse>, ServiceError> {
    let permission = Permission::new(Resource::User, Action::Update);
    match state.user_service.organization_of(user_id).await? {
        Some(org) => {
            state
                .authorization_service
                .require(user.id, permission, org)
                .await?
        }
        // Legacy users predate organization membership and have no org to scope
        // against; fall back to requiring user:update anywhere in the caller's scope.
        None => {
            let ctx = state.authorization_service.context_for(user.id).await?;
            if !ctx.permissions.allows(permission) {
                return Err(AuthError::Forbidden.into());
            }
        }
    }
    let profile = req.try_into_profile(user_id)?;
    let updated = state.user_service.update_profile(profile).await?;
    Ok(Json((&updated).into()))
}

#[utoipa::path(get, path = "/users/{user_id}/roles", tag = "Users",
    operation_id = "listUserRoles",
    summary = "List a user's assigned roles",
    params(("user_id" = Uuid, Path, description = "User id")),
    responses(
        (status = 200, description = "The user's roles", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(user.id = %user_id))]
pub async fn list_user_roles(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let views = state.role_service.roles_of_user(user_id).await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(post, path = "/users/{user_id}/roles", tag = "Users",
    operation_id = "assignUserRole",
    summary = "Assign a role to a user",
    description = "Requires user:update in the role's organization, plus a permission set that does not exceed the caller's own grants.",
    params(("user_id" = Uuid, Path, description = "User id")),
    request_body = AssignRoleRequest,
    responses(
        (status = 201, description = "Role assigned", body = RoleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 409, description = "Role is a template and cannot be assigned"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(user.id = %user_id))]
pub async fn assign_user_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), ServiceError> {
    let role_id = Id::new(req.role_id);
    let role = state.role_service.by_id(role_id).await?;
    if let Some(role_org) = role.organization_id {
        let perms: BTreeSet<Permission> = role.permissions.iter().copied().collect();
        state
            .authorization_service
            .require(
                user.id,
                Permission::new(Resource::User, Action::Update),
                role_org,
            )
            .await?;
        state
            .authorization_service
            .require_superset(user.id, &perms, role_org)
            .await?;
    }
    state.role_service.assign(user_id, role_id).await?;
    Ok((StatusCode::CREATED, Json((&role).into())))
}

#[utoipa::path(delete, path = "/users/{user_id}/roles/{role_id}", tag = "Users",
    operation_id = "revokeUserRole",
    summary = "Revoke a role from a user",
    description = "Requires user:update in the role's organization.",
    params(
        ("user_id" = Uuid, Path, description = "User id"),
        ("role_id" = Uuid, Path, description = "Role id"),
    ),
    responses(
        (status = 204, description = "Role revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(user.id = %user_id))]
pub async fn revoke_user_role(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ServiceError> {
    let role_id = Id::new(role_id);
    let role = state.role_service.by_id(role_id).await?;
    if let Some(role_org) = role.organization_id {
        state
            .authorization_service
            .require(
                user.id,
                Permission::new(Resource::User, Action::Update),
                role_org,
            )
            .await?;
    }
    state.role_service.revoke(user_id, role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/users/{user_id}/organization", tag = "Users",
    operation_id = "setUserOrganization",
    summary = "Set a user's organization",
    description = "Moves the user into the given organization. Requires user:update in the target organization.",
    params(("user_id" = Uuid, Path, description = "User id")),
    request_body = SetOrganizationRequest,
    responses(
        (status = 200, description = "Updated user", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(user.id = %user_id))]
pub async fn set_user_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SetOrganizationRequest>,
) -> Result<Json<UserResponse>, ServiceError> {
    let org = Id::new(req.organization_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::User, Action::Update),
            org,
        )
        .await?;
    let updated = state.user_service.set_organization(user_id, org).await?;
    Ok(Json((&updated).into()))
}
