use std::sync::Arc;

use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::extractors::{Json, Path, PluginPrincipal, Query};
use crate::http::v1::error::ErrorBody;
use crate::http::{AppState, auth::extractor::AuthUserExtractor};
use crate::service::{Feature, ServiceError, plugin_ingest_service::MAX_INGEST_ITEMS};
use domain::{
    authorization::{Action, Permission, Resource},
    plugin::{PluginSlug, PluginView},
};

use super::dto::plugin::{
    IngestBatchResponse, PluginCreateRequest, PluginKeyResponse, PluginResponse,
    PluginUpdateRequest, PluginViewResponse, TreeIngestBatchRequest, TreeRefListParams,
    TreeRefPageResponse,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_plugins, install_plugin))
        .routes(routes!(get_plugin, update_plugin, uninstall_plugin))
        .routes(routes!(rotate_plugin_key))
        .routes(routes!(get_plugin_view))
}

/// The ingest surface: authenticated with `PluginPrincipal` (an API key, not a
/// user session), so registered separately from [`routes`] to stay in the
/// public router while installation and management require a login.
pub fn ingest_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(get_own_plugin))
        .routes(routes!(list_tree_refs, upsert_trees))
        .routes(routes!(delete_tree_ref))
}

fn guard(state: &AppState) -> Result<(), ServiceError> {
    if !state.feature_flags.plugins_enabled {
        return Err(ServiceError::FeatureDisabled {
            feature: Feature::Plugins,
        });
    }
    Ok(())
}

#[utoipa::path(get, path = "/plugins", tag = "Plugins",
    operation_id = "listPlugins",
    summary = "List plugins visible to the caller",
    description = "Returns every installed plugin the caller may read, scoped to their visible organization subtree. Requires plugin:read.",
    responses(
        (status = 200, description = "Plugins visible to the caller", body = Vec<PluginResponse>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<Vec<PluginResponse>>, ServiceError> {
    guard(&state)?;
    let views = state.plugin_service.list(user.id).await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(post, path = "/plugins", tag = "Plugins",
    operation_id = "installPlugin",
    summary = "Install a plugin",
    description = "Creates a new plugin and returns its one-time plaintext API key. Requires plugin:create in the target organization, plus a permission set that does not exceed the caller's own grants.",
    request_body = PluginCreateRequest,
    responses(
        (status = 201, description = "Plugin installed; the plaintext key appears only in this response", body = PluginKeyResponse),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 409, description = "Slug already in use (code `resource.already_exists`)", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn install_plugin(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<PluginCreateRequest>,
) -> Result<(StatusCode, Json<PluginKeyResponse>), ServiceError> {
    guard(&state)?;
    let draft = req.into_draft(&state.app_origins)?;
    let (_, key) = state.plugin_service.install(user.id, draft).await?;
    Ok((StatusCode::CREATED, Json(PluginKeyResponse { key })))
}

#[utoipa::path(get, path = "/plugins/{plugin_slug}", tag = "Plugins",
    operation_id = "getPlugin",
    summary = "Get a plugin",
    description = "Returns plugin information by slug. Requires plugin:read in the plugin's organization.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 200, description = "Plugin found", body = PluginResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Plugin not found", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plugin.slug = %slug))]
pub async fn get_plugin(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(slug): Path<String>,
) -> Result<Json<PluginResponse>, ServiceError> {
    guard(&state)?;
    let slug = PluginSlug::new(slug)?;
    // `PluginService::by_slug` is unauthenticated by design (the ingest path
    // authenticates via the plugin's own key, not a user token), so this is
    // the one call site that must check read access itself before returning
    // anything — otherwise any caller who knows a slug could read a plugin
    // belonging to another organization.
    let view = state.plugin_service.by_slug(&slug).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Plugin, Action::Read),
            view.organization_id,
        )
        .await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(get, path = "/plugins/{plugin_slug}/view", tag = "Plugins",
    operation_id = "getPluginView",
    summary = "Get a plugin's view",
    description = "Returns what is needed to embed a plugin's view. Requires the plugin's own required_permissions in its organization -- plugin:read administers a plugin and is not what opening its view is about, though it grants access here as well.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 200, description = "Plugin view", body = PluginViewResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden, or the plugin is disabled (code `plugin.disabled`)", body = ErrorBody),
        (status = 404, description = "Plugin not found", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plugin.slug = %slug))]
pub async fn get_plugin_view(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(slug): Path<String>,
) -> Result<Json<PluginViewResponse>, ServiceError> {
    guard(&state)?;
    let slug = PluginSlug::new(slug)?;
    let view = state.plugin_service.view_for(user.id, &slug).await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(patch, path = "/plugins/{plugin_slug}", tag = "Plugins",
    operation_id = "updatePlugin",
    summary = "Update a plugin",
    description = "Applies the given fields to an installed plugin; an omitted field is left untouched. Requires plugin:update in the plugin's organization, plus (when permissions change) a set that does not exceed the caller's own grants.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    request_body = PluginUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = PluginResponse),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Plugin not found", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plugin.slug = %slug))]
pub async fn update_plugin(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(slug): Path<String>,
    Json(req): Json<PluginUpdateRequest>,
) -> Result<Json<PluginResponse>, ServiceError> {
    guard(&state)?;
    let slug = PluginSlug::new(slug)?;
    let change = req.into_change(&state.app_origins)?;
    let view = state.plugin_service.update(user.id, &slug, change).await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(delete, path = "/plugins/{plugin_slug}", tag = "Plugins",
    operation_id = "uninstallPlugin",
    summary = "Uninstall a plugin",
    description = "Removes an installed plugin. Requires plugin:delete in the plugin's organization.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 204, description = "Uninstalled"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Plugin not found", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plugin.slug = %slug))]
pub async fn uninstall_plugin(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(slug): Path<String>,
) -> Result<StatusCode, ServiceError> {
    guard(&state)?;
    let slug = PluginSlug::new(slug)?;
    state.plugin_service.uninstall(user.id, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/plugins/{plugin_slug}/key", tag = "Plugins",
    operation_id = "rotatePluginKey",
    summary = "Rotate a plugin's API key",
    description = "Invalidates the current key and returns a new plaintext key, which appears only in this response. Requires plugin:update in the plugin's organization.",
    params(("plugin_slug" = String, Path, description = "Plugin slug")),
    responses(
        (status = 200, description = "Key rotated; the plaintext key appears only in this response", body = PluginKeyResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Plugin not found", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plugin.slug = %slug))]
pub async fn rotate_plugin_key(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(slug): Path<String>,
) -> Result<Json<PluginKeyResponse>, ServiceError> {
    guard(&state)?;
    let slug = PluginSlug::new(slug)?;
    let key = state.plugin_service.rotate_key(user.id, &slug).await?;
    Ok(Json(PluginKeyResponse { key }))
}

#[utoipa::path(get, path = "/plugins/me", tag = "Plugins",
    operation_id = "getOwnPlugin",
    summary = "Get the authenticated plugin",
    description = "Returns the calling plugin's own registration, authenticated by its API key rather than a user session. Lets an adapter confirm its organization and rights before importing anything.",
    responses(
        (status = 200, description = "The authenticated plugin", body = PluginResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Plugin is disabled", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_own_plugin(
    State(state): State<Arc<AppState>>,
    plugin: PluginPrincipal,
) -> Result<Json<PluginResponse>, ServiceError> {
    guard(&state)?;
    let last_seen_at = state.plugin_reader.last_seen_at(plugin.0.id).await?;
    let view = PluginView::from_aggregate(&plugin.0, last_seen_at);
    Ok(Json((&view).into()))
}

#[utoipa::path(get, path = "/plugins/ingest/trees", tag = "Plugins",
    operation_id = "listPluginTreeRefs",
    summary = "List a plugin's own tree references",
    description = "Keyset page over the calling plugin's external_id-to-tree mappings, ordered by external_id. A plugin never sees another plugin's references.",
    params(TreeRefListParams),
    responses(
        (status = 200, description = "Page of tree references", body = TreeRefPageResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Plugin is disabled", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_tree_refs(
    State(state): State<Arc<AppState>>,
    plugin: PluginPrincipal,
    Query(params): Query<TreeRefListParams>,
) -> Result<Json<TreeRefPageResponse>, ServiceError> {
    guard(&state)?;
    let limit = params
        .limit
        .unwrap_or(100)
        .clamp(1, MAX_INGEST_ITEMS as u32);
    let page = state
        .plugin_ingest_service
        .list_refs(&plugin.0, params.cursor.as_deref(), limit)
        .await?;
    Ok(Json(page.into()))
}

#[utoipa::path(post, path = "/plugins/ingest/trees", tag = "Plugins",
    operation_id = "upsertPluginTrees",
    summary = "Upsert trees from a plugin",
    description = "Creates or updates up to 500 trees per request, matched by the plugin's own external_id. Every entry is processed independently: one bad entry fails only itself, never the batch. Requires tree:create and tree:update in the plugin's organization.",
    request_body = TreeIngestBatchRequest,
    responses(
        (status = 200, description = "Per-item results plus a summary", body = IngestBatchResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden or plugin disabled", body = ErrorBody),
        (status = 413, description = "Batch too large (code `plugin.batch_too_large`)", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn upsert_trees(
    State(state): State<Arc<AppState>>,
    plugin: PluginPrincipal,
    Json(body): Json<TreeIngestBatchRequest>,
) -> Result<Json<IngestBatchResponse>, ServiceError> {
    guard(&state)?;
    if body.items.len() > MAX_INGEST_ITEMS {
        return Err(ServiceError::PayloadTooLarge {
            limit: MAX_INGEST_ITEMS,
        });
    }
    let items = body.items.into_iter().map(|i| i.into_item()).collect();
    let results = state
        .plugin_ingest_service
        .upsert_trees(&plugin.0, items)
        .await?;
    Ok(Json(results.into()))
}

#[utoipa::path(delete, path = "/plugins/ingest/trees/{external_id}", tag = "Plugins",
    operation_id = "deletePluginTree",
    summary = "Delete a tree linked by a plugin",
    description = "Deletes the tree the plugin's external_id resolves to, through the same path as the regular tree deletion so cluster centroid and status are recalculated. Requires tree:delete in the plugin's organization.",
    params(("external_id" = String, Path, description = "The plugin's own external identifier for the tree")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden or plugin disabled", body = ErrorBody),
        (status = 404, description = "No tree is linked under this external_id", body = ErrorBody),
        (status = 503, description = "Plugins feature is disabled (code `feature.plugins_disabled`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(external_id = %external_id))]
pub async fn delete_tree_ref(
    State(state): State<Arc<AppState>>,
    plugin: PluginPrincipal,
    Path(external_id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    guard(&state)?;
    state
        .plugin_ingest_service
        .delete_tree(&plugin.0, &external_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
