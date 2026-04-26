use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use axum::routing::{get, post};
use utoipa_axum::router::OpenApiRouter;

use crate::http::AppState;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/login", get(login))
        .route("/users/login/token", post(login_token))
        .route("/users/logout", post(logout))
        .route("/users/role/{role_id}", get(list_users_by_role))
        .route("/users/token/refresh", post(refresh_token))
}

pub async fn list_users(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn create_user(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn login(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn login_token(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn logout(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}

pub async fn list_users_by_role(
    State(_state): State<Arc<AppState>>,
    Path(_role_id): Path<String>,
) -> Json<()> {
    todo!()
}

pub async fn refresh_token(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}
