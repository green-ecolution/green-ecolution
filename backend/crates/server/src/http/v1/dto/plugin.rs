use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;
use uuid::Uuid;

use domain::{
    Id,
    authorization::ResourceCatalog,
    plugin::{
        DeviceCapability, Plugin, PluginDraft, PluginFrontend, PluginName, PluginSlug, PluginView,
        ServiceEndpoint,
    },
    sensor::SensorDraft,
};

use crate::http::AppOrigins;
use crate::service::{Malformed, ServiceError, plugin_service::PluginChange};

use super::role::parse_permissions;
use super::sensor::{CreateSensorRequest, LorawanCredentialsRequest, SensorTypeResponse};

fn parse_device_capabilities(raw: &[String]) -> Result<BTreeSet<DeviceCapability>, ServiceError> {
    raw.iter()
        .map(|c| c.parse::<DeviceCapability>().map_err(ServiceError::from))
        .collect()
}

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
    "device_capabilities": [],
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
    pub device_capabilities: Vec<String>,
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
            device_capabilities: view.device_capabilities.clone(),
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
    "frontend_target": "https://kataster.example.org/view",
    "device_capabilities": ["camera", "bluetooth"],
    "view_ticket": "gev_3f2a9c41d7b05e6a8f1c2d3e4b5a69788796a5b4c3d2e1f00112233445566778",
    "frontend_url": null
}))]
pub struct PluginViewResponse {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub frontend_mode: String,
    pub frontend_target: Option<String>,
    pub device_capabilities: Vec<String>,
    /// Single-use, valid for two minutes. Hand it to the plugin's own backend,
    /// which redeems it to learn who opened the view.
    pub view_ticket: String,
    /// Where the iframe starts for a `proxied` plugin: the session URL on the
    /// plugin's own host, carrying this response's ticket. `None` otherwise.
    pub frontend_url: Option<String>,
}

impl PluginViewResponse {
    pub fn new(view: &PluginView, view_ticket: String, frontend_url: Option<String>) -> Self {
        Self {
            slug: view.slug.clone(),
            name: view.name.clone(),
            description: view.description.clone(),
            frontend_mode: view.frontend_mode.to_string(),
            frontend_target: view.frontend_target.clone(),
            device_capabilities: view.device_capabilities.clone(),
            view_ticket,
            frontend_url,
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
    #[serde(default)]
    pub device_capabilities: Vec<String>,
    pub frontend: PluginFrontendDto,
}

impl PluginCreateRequest {
    pub fn into_draft(
        self,
        app_origins: &AppOrigins,
        catalog: &ResourceCatalog,
    ) -> Result<PluginDraft, ServiceError> {
        Ok(PluginDraft {
            slug: PluginSlug::new(self.slug)?,
            name: PluginName::new(self.name)?,
            description: self.description,
            organization_id: Id::new(self.organization_id),
            permissions: parse_permissions(&self.permissions, catalog)?,
            required_permissions: parse_permissions(&self.required_permissions, catalog)?,
            device_capabilities: parse_device_capabilities(&self.device_capabilities)?,
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
    pub device_capabilities: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

impl PluginUpdateRequest {
    pub fn into_change(
        self,
        app_origins: &AppOrigins,
        catalog: &ResourceCatalog,
    ) -> Result<PluginChange, ServiceError> {
        Ok(PluginChange {
            name: self.name.map(PluginName::new).transpose()?,
            description: self.description,
            frontend: self
                .frontend
                .map(|frontend| frontend.into_domain(app_origins))
                .transpose()?,
            permissions: self
                .permissions
                .map(|p| parse_permissions(&p, catalog))
                .transpose()?,
            required_permissions: self
                .required_permissions
                .map(|p| parse_permissions(&p, catalog))
                .transpose()?,
            device_capabilities: self
                .device_capabilities
                .map(|c| parse_device_capabilities(&c))
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "ticket": "gev_3f2a…" }))]
pub struct ViewTicketRedeemRequest {
    pub ticket: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ViewTicketUserResponse {
    pub id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "user": { "id": "01990000-0000-7000-8000-0000000000aa", "display_name": "Jane Doe" },
    "organization_id": "01980000-0000-7000-8000-000000000001"
}))]
pub struct ViewTicketRedeemResponse {
    pub user: ViewTicketUserResponse,
    pub organization_id: Uuid,
}

/// One navigation entry: just enough to link to `/plugin/{slug}`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "slug": "sensor-setup", "name": "Sensor-Einrichtung" }))]
pub struct PluginNavEntryResponse {
    pub slug: String,
    pub name: String,
}

impl From<&PluginView> for PluginNavEntryResponse {
    fn from(view: &PluginView) -> Self {
        Self {
            slug: view.slug.clone(),
            name: view.name.clone(),
        }
    }
}

/// Body for `POST /plugins/ingest/sensors`: the user-facing create request
/// without `organization_id` and `provider`, which always come from the plugin.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PluginSensorCreateRequest {
    #[schema(example = "eui-a84041000181c001")]
    pub id: String,
    pub sensor_type: SensorTypeResponse,
    pub model_id: Uuid,
    #[serde(default)]
    #[schema(value_type = Option<Object>, nullable)]
    pub additional_information: Option<serde_json::Value>,
    #[serde(default)]
    #[schema(nullable)]
    pub lorawan: Option<LorawanCredentialsRequest>,
}

impl PluginSensorCreateRequest {
    pub fn into_draft(self, plugin: &Plugin) -> Result<SensorDraft, ServiceError> {
        CreateSensorRequest {
            id: self.id,
            sensor_type: self.sensor_type,
            model_id: self.model_id,
            provider: Some(plugin.slug().as_str().to_string()),
            additional_information: self.additional_information,
            lorawan: self.lorawan,
            organization_id: None,
        }
        .into_draft(plugin.organization_id())
    }
}
