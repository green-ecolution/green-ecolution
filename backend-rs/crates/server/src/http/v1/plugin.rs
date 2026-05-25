use std::sync::Arc;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::dto::{
            plugin::{
                PluginAuthRequest, PluginListResponse, PluginRegisterRequest, PluginResponse,
            },
            user::ClientTokenResponse,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_plugins, register_plugin))
        .routes(routes!(get_plugin))
        .routes(routes!(plugin_heartbeat))
        .routes(routes!(plugin_refresh_token))
        .routes(routes!(unregister_plugin))
}

fn guard(state: &AppState) -> Result<(), ServiceError> {
    if !state.feature_flags.plugins_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "plugins" });
    }
    Ok(())
}

#[utoipa::path(get, path = "/plugins", tag = "Plugins",
    operation_id = "listPlugins",
    summary = "List all plugins",
    description = "Returns all registered plugins.",
    responses(
        (status = 200, description = "List of registered plugins", body = PluginListResponse),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PluginListResponse>, ServiceError> {
    guard(&state)?;
    todo!()
}

#[utoipa::path(post, path = "/plugins", tag = "Plugins",
    operation_id = "registerPlugin",
    summary = "Register a plugin",
    description = "Register a new external plugin and receive authentication tokens.",
    request_body = PluginRegisterRequest,
    responses(
        (status = 201, description = "Plugin registered", body = ClientTokenResponse),
        (status = 400, description = "Invalid input"),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn register_plugin(
    State(state): State<Arc<AppState>>,
    _body: Bytes,
) -> Result<(StatusCode, Json<ClientTokenResponse>), ServiceError> {
    guard(&state)?;
    todo!()
}

#[utoipa::path(get, path = "/plugins/{plugin_slug}", tag = "Plugins",
    operation_id = "getPlugin",
    summary = "Get a plugin",
    description = "Returns plugin information by slug.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 200, description = "Plugin found", body = PluginResponse),
        (status = 404, description = "Plugin not found"),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_plugin(
    State(state): State<Arc<AppState>>,
    Path(_slug): Path<String>,
) -> Result<Json<PluginResponse>, ServiceError> {
    guard(&state)?;
    todo!()
}

#[utoipa::path(post, path = "/plugins/{plugin_slug}/heartbeat", tag = "Plugins",
    operation_id = "pluginHeartbeat",
    summary = "Send heartbeat",
    description = "Send a keepalive heartbeat for a registered plugin.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 200, description = "Heartbeat received"),
        (status = 404, description = "Plugin not found"),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn plugin_heartbeat(
    State(state): State<Arc<AppState>>,
    Path(_slug): Path<String>,
) -> Result<Json<String>, ServiceError> {
    guard(&state)?;
    todo!()
}

#[utoipa::path(post, path = "/plugins/{plugin_slug}/token/refresh", tag = "Plugins",
    operation_id = "refreshPluginToken",
    summary = "Refresh plugin token",
    description = "Refresh authentication token for a plugin.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    request_body = PluginAuthRequest,
    responses(
        (status = 200, description = "Token refreshed", body = ClientTokenResponse),
        (status = 404, description = "Plugin not found"),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn plugin_refresh_token(
    State(state): State<Arc<AppState>>,
    Path(_slug): Path<String>,
    _body: Bytes,
) -> Result<Json<ClientTokenResponse>, ServiceError> {
    guard(&state)?;
    todo!()
}

#[utoipa::path(post, path = "/plugins/{plugin_slug}/unregister", tag = "Plugins",
    operation_id = "unregisterPlugin",
    summary = "Unregister a plugin",
    description = "Remove a plugin registration.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 204, description = "Plugin unregistered"),
        (status = 404, description = "Plugin not found"),
        (status = 503, description = "Plugins feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn unregister_plugin(
    State(state): State<Arc<AppState>>,
    Path(_slug): Path<String>,
) -> Result<StatusCode, ServiceError> {
    guard(&state)?;
    todo!()
}
