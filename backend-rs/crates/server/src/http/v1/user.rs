use std::{str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                user::{UserRegisterRequest, UserResponse},
            },
            pagination::PaginationParams,
        },
    },
    require_role,
    service::ServiceError,
};
use domain::{
    shared::pagination::Pagination,
    user::{UserCreate as DomainUserCreate, UserRole as DomainUserRole},
};

pub fn protected_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_users, create_user))
        .routes(routes!(list_users_by_role))
}

#[utoipa::path(get, path = "/users", tag = "Users",
    operation_id = "listUsers",
    summary = "List all users",
    description = "Returns a paginated list of registered users.",
    params(PaginationParams),
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
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state.user_service.list(pagination).await?;
    Ok(Json(ListResponse::from_page(page, &pagination)))
}

#[utoipa::path(post, path = "/users", tag = "Users",
    operation_id = "createUser",
    summary = "Register a new user",
    description = "Admin-only user registration.",
    request_body = UserRegisterRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<UserRegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ServiceError> {
    require_role!(user, DomainUserRole::Tbz, DomainUserRole::GreenEcolution);
    let entity: DomainUserCreate = req.try_into()?;
    let created = state.user_service.register(entity).await?;
    Ok((StatusCode::CREATED, Json((&created).into())))
}

#[utoipa::path(get, path = "/users/role/{role_id}", tag = "Users",
    operation_id = "listUsersByRole",
    summary = "List users by role",
    description = "Returns a filtered list of users with a specific role.",
    params(
        ("role_id" = String, Path, description = "User role"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Users with role", body = ListResponse<UserResponse>),
        (status = 400, description = "Unknown role"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_users_by_role(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(role_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>, ServiceError> {
    let role = DomainUserRole::from_str(&role_id)
        .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
    let pagination = Pagination::from(&params);
    let page = state.user_service.by_role(role, pagination).await?;
    Ok(Json(ListResponse::from_page(page, &pagination)))
}
