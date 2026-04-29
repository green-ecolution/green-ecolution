use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                user::{
                    ClientTokenResponse, LoginResponse, LoginTokenRequest, LogoutRequest,
                    RefreshTokenRequest, UserRegisterRequest, UserResponse,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_users, create_user))
        .routes(routes!(login))
        .routes(routes!(login_token))
        .routes(routes!(logout))
        .routes(routes!(list_users_by_role))
        .routes(routes!(refresh_token))
}

#[utoipa::path(get, path = "/users", tag = "Users",
    operation_id = "listUsers",
    summary = "List all users",
    description = "Returns a paginated list of registered users.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of users", body = ListResponse<UserResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_users(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>, ServiceError> {
    todo!()
}

#[utoipa::path(post, path = "/users", tag = "Users",
    operation_id = "createUser",
    summary = "Register a new user",
    description = "Admin-only user registration.",
    request_body = UserRegisterRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_user(
    State(_state): State<Arc<AppState>>,
    Json(_entity): Json<UserRegisterRequest>,
) -> Result<Json<UserResponse>, ServiceError> {
    todo!()
}

#[utoipa::path(get, path = "/users/login", tag = "Users",
    operation_id = "loginUser",
    summary = "Initiate login",
    description = "Initiate OAuth2/OIDC login flow.",
    params(("redirect_url" = String, Query, description = "Redirect URL after login")),
    responses(
        (status = 200, description = "Login URL", body = LoginResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn login(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<LoginResponse>, ServiceError> {
    todo!()
}

#[utoipa::path(post, path = "/users/login/token", tag = "Users",
    operation_id = "exchangeLoginToken",
    summary = "Exchange auth code for tokens",
    description = "Exchange an authorization code for access/refresh tokens.",
    params(("redirect_url" = String, Query, description = "Redirect URL")),
    request_body = LoginTokenRequest,
    responses(
        (status = 200, description = "Token response", body = ClientTokenResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn login_token(
    State(_state): State<Arc<AppState>>,
    Json(_entity): Json<LoginTokenRequest>,
) -> Result<Json<ClientTokenResponse>, ServiceError> {
    todo!()
}

#[utoipa::path(post, path = "/users/logout", tag = "Users",
    operation_id = "logoutUser",
    summary = "Logout user",
    description = "Invalidate a user session.",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn logout(
    State(_state): State<Arc<AppState>>,
    Json(_entity): Json<LogoutRequest>,
) -> Result<Json<String>, ServiceError> {
    todo!()
}

#[utoipa::path(get, path = "/users/role/{role_id}", tag = "Users",
    operation_id = "listUsersByRole",
    summary = "List users by role",
    description = "Returns a filtered list of users with a specific role.",
    params(("role_id" = String, Path, description = "User role")),
    responses(
        (status = 200, description = "Users with role", body = ListResponse<UserResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_users_by_role(
    State(_state): State<Arc<AppState>>,
    Path(_role_id): Path<String>,
) -> Result<Json<ListResponse<UserResponse>>, ServiceError> {
    todo!()
}

#[utoipa::path(post, path = "/users/token/refresh", tag = "Users",
    operation_id = "refreshToken",
    summary = "Refresh access token",
    description = "Obtain a new access token using a refresh token.",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Refreshed token", body = ClientTokenResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn refresh_token(
    State(_state): State<Arc<AppState>>,
    Json(_entity): Json<RefreshTokenRequest>,
) -> Result<Json<ClientTokenResponse>, ServiceError> {
    todo!()
}
