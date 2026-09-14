use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;
use uuid::Uuid;

use domain::{
    Id,
    plugin::{
        PluginDraft, PluginFrontend, PluginName, PluginSlug, PluginView, ServiceEndpoint, TreeRef,
        TreeRefPage,
    },
};

use crate::http::AppOrigins;
use crate::service::{
    Malformed, ServiceError,
    plugin_ingest_service::{IngestResult, IngestStatus, TreeIngestItem},
    plugin_service::PluginChange,
};

use super::role::parse_permissions;

/// Represents an installed plugin, mirroring `PluginView` flat: no hash, no
/// key. A later task reads `frontend_target` directly off this response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "01990000-0000-7000-8000-000000000001",
    "slug": "tbz-baumkataster",
    "name": "TBZ Baumkataster",
    "description": null,
    "organization_id": "01980000-0000-7000-8000-000000000001",
    "permissions": ["tree:create", "tree:update"],
    "required_permissions": ["tree:read"],
    "frontend_mode": "none",
    "frontend_target": null,
    "enabled": true,
    "has_credential": true,
    "last_seen_at": "2026-09-01T10:00:00Z",
    "created_at": "2026-08-01T00:00:00Z"
}))]
pub struct PluginResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub permissions: Vec<String>,
    pub required_permissions: Vec<String>,
    pub frontend_mode: String,
    pub frontend_target: Option<String>,
    pub enabled: bool,
    pub has_credential: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<&PluginView> for PluginResponse {
    fn from(view: &PluginView) -> Self {
        Self {
            id: view.id.value(),
            slug: view.slug.clone(),
            name: view.name.clone(),
            description: view.description.clone(),
            organization_id: view.organization_id.value(),
            permissions: view.permissions.clone(),
            required_permissions: view.required_permissions.clone(),
            frontend_mode: view.frontend_mode.to_string(),
            frontend_target: view.frontend_target.clone(),
            enabled: view.enabled,
            has_credential: view.has_credential,
            last_seen_at: view.last_seen_at,
            created_at: view.created_at,
        }
    }
}

/// What the plugin viewer needs to render a frame, and nothing else. Its
/// reader holds the plugin's `required_permissions`, not `plugin:read`, so it
/// deliberately omits the administrative facts of [`PluginResponse`] — the
/// plugin's own permissions, its credential state and its last sign of life.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "tbz-baumkataster",
    "name": "TBZ Baumkataster",
    "description": null,
    "frontend_mode": "external",
    "frontend_target": "https://kataster.example.org/view"
}))]
pub struct PluginViewResponse {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub frontend_mode: String,
    pub frontend_target: Option<String>,
}

impl From<&PluginView> for PluginViewResponse {
    fn from(view: &PluginView) -> Self {
        Self {
            slug: view.slug.clone(),
            name: view.name.clone(),
            description: view.description.clone(),
            frontend_mode: view.frontend_mode.to_string(),
            frontend_target: view.frontend_target.clone(),
        }
    }
}

/// One-time response carrying the plaintext API key, issued on install and
/// on rotation only — it never appears again afterwards.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "key": "gep_0199a1f0-4c3e-7b2d-8a41-6f5c2e9d3b70.7b1e3c9a4f2d6081b3c5e7a9d1f3b5c72e4a6c8e0f1d3b5a7c9e1f3d5b7a9c1e" }))]
pub struct PluginKeyResponse {
    pub key: String,
}

/// Frontend wiring for a plugin, tagged so mode and target cannot be set
/// inconsistently.
///
/// Variant order is load-bearing for the generated TypeScript client, not a
/// matter of taste. utoipa emits the variants as a `oneOf` in declaration
/// order and openapi-generator turns that into a first-match dispatch whose
/// guards only test which properties are present. `None` carries nothing, so
/// its guard is `"mode" in value` — true for every variant. Listed first it
/// swallows the other two and serializes them without their `target`, which
/// silently drops the plugin's address. Payload-carrying variants therefore
/// come first. `External` and `Proxied` share a shape and stay
/// indistinguishable to those guards, but both emit `{ mode, target }`
/// verbatim, so the wire format is right either way.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum PluginFrontendDto {
    External { target: String },
    Proxied { target: String },
    None,
}

impl PluginFrontendDto {
    fn into_domain(self, app_origins: &AppOrigins) -> Result<PluginFrontend, ServiceError> {
        match self {
            Self::None => Ok(PluginFrontend::None),
            Self::External { target } => {
                let url = Url::parse(&target).map_err(|e| ServiceError::Malformed {
                    kind: Malformed::PluginFrontend,
                    detail: format!("invalid url: {e}"),
                })?;
                // localhost is the one exception, for local development against
                // a plugin that has no certificate yet.
                let is_localhost = url.host_str() == Some("localhost");
                if url.scheme() != "https" && !is_localhost {
                    return Err(ServiceError::Malformed {
                        kind: Malformed::PluginFrontend,
                        detail:
                            "external frontend target must be an absolute https url (localhost excepted)"
                                .into(),
                    });
                }
                // The iframe sandbox keeps `allow-same-origin` safe only because the
                // plugin's document sits on a foreign origin; the app's own origin
                // would give it unrestricted script access to the app's DOM and storage.
                if app_origins.contains(&url) {
                    return Err(ServiceError::Malformed {
                        kind: Malformed::PluginFrontend,
                        detail: "external frontend target must not be the application's own origin"
                            .into(),
                    });
                }
                Ok(PluginFrontend::External(url))
            }
            Self::Proxied { target } => {
                let (host, port) =
                    target
                        .rsplit_once(':')
                        .ok_or_else(|| ServiceError::Malformed {
                            kind: Malformed::PluginFrontend,
                            detail: "proxied frontend target must be host:port".into(),
                        })?;
                let port: u16 = port.parse().map_err(|_| ServiceError::Malformed {
                    kind: Malformed::PluginFrontend,
                    detail: "proxied frontend target must carry a numeric port".into(),
                })?;
                // The service allowlist check arrives with the proxy plan; not
                // required here yet.
                Ok(PluginFrontend::Proxied(ServiceEndpoint::new(host, port)?))
            }
        }
    }
}

/// Request body for installing a new plugin.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "tbz-baumkataster",
    "name": "TBZ Baumkataster",
    "organization_id": "01980000-0000-7000-8000-000000000001",
    "permissions": ["tree:create"],
    "required_permissions": ["tree:read"],
    "frontend": { "mode": "none" }
}))]
pub struct PluginCreateRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub permissions: Vec<String>,
    pub required_permissions: Vec<String>,
    pub frontend: PluginFrontendDto,
}

impl PluginCreateRequest {
    pub fn into_draft(self, app_origins: &AppOrigins) -> Result<PluginDraft, ServiceError> {
        Ok(PluginDraft {
            slug: PluginSlug::new(self.slug)?,
            name: PluginName::new(self.name)?,
            description: self.description,
            organization_id: Id::new(self.organization_id),
            permissions: parse_permissions(&self.permissions)?,
            required_permissions: parse_permissions(&self.required_permissions)?,
            frontend: self.frontend.into_domain(app_origins)?,
        })
    }
}

/// Request body for updating an installed plugin. Every field is optional; an
/// omitted field leaves it untouched. `description` is a double-option so
/// "omitted" and "set to null" (clear it) stay distinguishable, matching
/// `PluginChange`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "enabled": true }))]
pub struct PluginUpdateRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[schema(value_type = Option<String>)]
    pub description: Option<Option<String>>,
    pub frontend: Option<PluginFrontendDto>,
    pub permissions: Option<Vec<String>>,
    pub required_permissions: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

impl PluginUpdateRequest {
    pub fn into_change(self, app_origins: &AppOrigins) -> Result<PluginChange, ServiceError> {
        Ok(PluginChange {
            name: self.name.map(PluginName::new).transpose()?,
            description: self.description,
            frontend: self
                .frontend
                .map(|frontend| frontend.into_domain(app_origins))
                .transpose()?,
            permissions: self
                .permissions
                .map(|p| parse_permissions(&p))
                .transpose()?,
            required_permissions: self
                .required_permissions
                .map(|p| parse_permissions(&p))
                .transpose()?,
            enabled: self.enabled,
        })
    }
}

fn deserialize_some<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
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
