use std::{str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
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
    require_role,
    service::ServiceError,
};
use domain::{
    shared::pagination::Pagination,
    user::{UserCreate as DomainUserCreate, UserRole as DomainUserRole},
};

pub fn public_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(login_token))
        .routes(routes!(logout))
        .routes(routes!(refresh_token))
}

pub fn protected_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_users, create_user))
        .routes(routes!(list_users_by_role))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct LoginQuery {
    /// Where Keycloak should redirect to after a successful login.
    #[serde(default)]
    pub redirect_url: Option<String>,
    /// PKCE code challenge (base64url(sha256(verifier))). When present the
    /// backend appends `code_challenge_method=S256` to the auth URL.
    #[serde(default)]
    pub code_challenge: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct LoginTokenQuery {
    /// The same redirect URL the SPA used in the initial /login call.
    pub redirect_url: String,
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

#[utoipa::path(get, path = "/users/login", tag = "Users",
    operation_id = "loginUser",
    summary = "Initiate login",
    description = "Initiate OAuth2/OIDC login flow.",
    params(LoginQuery),
    responses(
        (status = 200, description = "Login URL", body = LoginResponse),
        (status = 400, description = "Invalid redirect_url"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginQuery>,
) -> Result<Json<LoginResponse>, ServiceError> {
    let redirect_url = query
        .redirect_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(url::Url::parse)
        .transpose()
        .map_err(|e| ServiceError::InvalidInput(format!("redirect_url: {e}")))?;
    let response = state
        .auth_service
        .login_url(redirect_url, query.code_challenge.as_deref());
    Ok(Json((&response).into()))
}

#[utoipa::path(post, path = "/users/login/token", tag = "Users",
    operation_id = "exchangeLoginToken",
    summary = "Exchange auth code for tokens",
    description = "Exchange an authorization code for access/refresh tokens.",
    params(LoginTokenQuery),
    request_body = LoginTokenRequest,
    responses(
        (status = 200, description = "Token response", body = ClientTokenResponse),
        (status = 400, description = "Invalid code or redirect_url"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn login_token(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginTokenQuery>,
    Json(req): Json<LoginTokenRequest>,
) -> Result<Json<ClientTokenResponse>, ServiceError> {
    let redirect_url = url::Url::parse(&query.redirect_url)
        .map_err(|e| ServiceError::InvalidInput(format!("redirect_url: {e}")))?;
    let token = state
        .auth_service
        .exchange_code(&req.code, redirect_url, req.code_verifier.as_deref())
        .await?;
    Ok(Json((&token).into()))
}

#[utoipa::path(post, path = "/users/logout", tag = "Users",
    operation_id = "logoutUser",
    summary = "Logout user",
    description = "Invalidate a user session.",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "Logged out"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, ServiceError> {
    state
        .user_service
        .revoke_session(&req.refresh_token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
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

#[utoipa::path(post, path = "/users/token/refresh", tag = "Users",
    operation_id = "refreshToken",
    summary = "Refresh access token",
    description = "Obtain a new access token using a refresh token.",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Refreshed token", body = ClientTokenResponse),
        (status = 400, description = "Invalid refresh token"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ClientTokenResponse>, ServiceError> {
    let token = state.auth_service.refresh(&req.refresh_token).await?;
    Ok(Json((&token).into()))
}
