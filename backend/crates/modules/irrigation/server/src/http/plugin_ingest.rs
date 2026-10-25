//! The tree half of the plugin ingest surface. Authenticated with
//! `PluginPrincipal` (an API key, not a user session), so it is mounted with
//! the public routes.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use irrigation_domain::plugin_refs::{TreeRef, TreeRefPage};

use super::state::IrrigationHttpState;
use crate::service::plugin_ingest_service::{
    IngestResult, IngestStatus, MAX_INGEST_ITEMS, TreeIngestItem,
};
use server::{
    http::{
        extractors::{Json, Path, PluginPrincipal, Query},
        v1::error::ErrorBody,
    },
    service::{Feature, ServiceError},
};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new()
        .routes(routes!(list_tree_refs, upsert_trees))
        .routes(routes!(delete_tree_ref))
}

fn guard(state: &IrrigationHttpState) -> Result<(), ServiceError> {
    if !state.plugin_auth.enabled() {
        return Err(ServiceError::FeatureDisabled {
            feature: Feature::Plugins,
        });
    }
    Ok(())
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
    State(state): State<Arc<IrrigationHttpState>>,
    plugin: PluginPrincipal,
    Query(params): Query<TreeRefListParams>,
) -> Result<Json<TreeRefPageResponse>, ServiceError> {
    guard(&state)?;
    let limit = params
        .limit
        .unwrap_or(100)
        .clamp(1, MAX_INGEST_ITEMS as u32);
    let page = state
        .plugin_ingest
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
    State(state): State<Arc<IrrigationHttpState>>,
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
    let results = state.plugin_ingest.upsert_trees(&plugin.0, items).await?;
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
    State(state): State<Arc<IrrigationHttpState>>,
    plugin: PluginPrincipal,
    Path(external_id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    guard(&state)?;
    state
        .plugin_ingest
        .delete_tree(&plugin.0, &external_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// One entry of a tree ingest batch. Field-by-field validation (species,
/// number, planting year, coordinate, ...) happens in the service, per item,
/// so a single malformed entry cannot fail the whole batch.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "external_id": "12345",
    "number": "FL-001",
    "species": "Quercus robur",
    "planting_year": 1998,
    "latitude": 54.7836,
    "longitude": 9.4321,
    "description": null,
    "additional_info": { "objectid": 12345 }
}))]
pub struct TreeIngestItemRequest {
    pub external_id: String,
    pub number: String,
    pub species: String,
    #[schema(minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[serde(default)]
    pub description: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_info: Option<serde_json::Value>,
}

impl TreeIngestItemRequest {
    pub fn into_item(self) -> TreeIngestItem {
        TreeIngestItem {
            external_id: self.external_id,
            number: self.number,
            species: self.species,
            planting_year: self.planting_year,
            latitude: self.latitude,
            longitude: self.longitude,
            description: self.description,
            additional_info: self.additional_info,
        }
    }
}

/// Request body for the tree ingest batch endpoint. Capped at
/// `plugin_ingest_service::MAX_INGEST_ITEMS` entries; a larger batch is
/// rejected wholesale (413) before any entry is processed.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeIngestBatchRequest {
    pub items: Vec<TreeIngestItemRequest>,
}

/// Outcome of one batch entry. `tree_id` is present on every non-`failed`
/// result, `error` only on `failed`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "external_id": "12345", "status": "created", "tree_id": "01990000-0000-7000-8000-000000000001" }))]
pub struct IngestResultResponse {
    pub external_id: String,
    pub status: IngestStatusResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IngestStatusResponse {
    Created,
    Updated,
    Unchanged,
    Failed,
}

impl From<IngestStatus> for IngestStatusResponse {
    fn from(status: IngestStatus) -> Self {
        match status {
            IngestStatus::Created => Self::Created,
            IngestStatus::Updated => Self::Updated,
            IngestStatus::Unchanged => Self::Unchanged,
            IngestStatus::Failed => Self::Failed,
        }
    }
}

impl From<IngestResult> for IngestResultResponse {
    fn from(result: IngestResult) -> Self {
        Self {
            external_id: result.external_id,
            status: result.status.into(),
            tree_id: result.tree_id.map(|id| id.value()),
            error: result.error,
        }
    }
}

/// Per-status counts across a batch, so a caller need not tally `results`
/// itself.
#[derive(Debug, Default, Serialize, utoipa::ToSchema)]
pub struct IngestSummaryResponse {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub failed: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IngestBatchResponse {
    pub results: Vec<IngestResultResponse>,
    pub summary: IngestSummaryResponse,
}

impl From<Vec<IngestResult>> for IngestBatchResponse {
    fn from(results: Vec<IngestResult>) -> Self {
        let mut summary = IngestSummaryResponse::default();
        for result in &results {
            match result.status {
                IngestStatus::Created => summary.created += 1,
                IngestStatus::Updated => summary.updated += 1,
                IngestStatus::Unchanged => summary.unchanged += 1,
                IngestStatus::Failed => summary.failed += 1,
            }
        }
        Self {
            results: results.into_iter().map(Into::into).collect(),
            summary,
        }
    }
}

/// One external_id-to-tree mapping owned by the calling plugin.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeRefResponse {
    pub external_id: String,
    pub tree_id: Uuid,
}

impl From<&TreeRef> for TreeRefResponse {
    fn from(r: &TreeRef) -> Self {
        Self {
            external_id: r.external_id.clone(),
            tree_id: r.tree_id.value(),
        }
    }
}

/// Keyset page over a plugin's own tree references, ordered by external_id.
/// No `total`: the adapter walks this to the end anyway, and a cursor stays
/// stable while imports run concurrently.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeRefPageResponse {
    pub items: Vec<TreeRefResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl From<TreeRefPage> for TreeRefPageResponse {
    fn from(page: TreeRefPage) -> Self {
        Self {
            items: page.items.iter().map(Into::into).collect(),
            next_cursor: page.next_cursor,
        }
    }
}

/// Query parameters for the tree reference listing endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TreeRefListParams {
    #[param(minimum = 1, example = 100)]
    pub limit: Option<u32>,
    #[param(example = "12344")]
    pub cursor: Option<String>,
}
