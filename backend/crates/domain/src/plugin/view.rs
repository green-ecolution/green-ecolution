use chrono::{DateTime, Utc};

use crate::plugin::{Plugin, PluginFrontend};

/// Read model for HTTP responses. Deliberately carries no key hash.
///
/// `created_at` derives from the UUID v7 `id` (no DB column), like every
/// other view in this codebase — `None` only for legacy non-v7 ids.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginView {
    pub id: crate::Id<Plugin>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: crate::Id<crate::organization::Organization>,
    pub permissions: Vec<String>,
    pub required_permissions: Vec<String>,
    pub frontend_mode: &'static str,
    pub frontend_target: Option<String>,
    pub enabled: bool,
    pub has_credential: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl PluginView {
    pub fn from_aggregate(plugin: &Plugin, last_seen_at: Option<DateTime<Utc>>) -> Self {
        let (frontend_mode, frontend_target) = match plugin.frontend() {
            PluginFrontend::None => ("none", None),
            PluginFrontend::External(url) => ("external", Some(url.to_string())),
            PluginFrontend::Proxied(ep) => {
                ("proxied", Some(format!("{}:{}", ep.host(), ep.port())))
            }
        };
        Self {
            id: plugin.id,
            slug: plugin.slug().as_str().to_string(),
            name: plugin.name.as_str().to_string(),
            description: plugin.description.clone(),
            organization_id: plugin.organization_id(),
            permissions: plugin.permissions().iter().map(|p| p.to_string()).collect(),
            required_permissions: plugin
                .required_permissions()
                .iter()
                .map(|p| p.to_string())
                .collect(),
            frontend_mode,
            frontend_target,
            enabled: plugin.enabled(),
            has_credential: plugin.key_hash().is_some(),
            last_seen_at,
            created_at: plugin.id.created_at(),
        }
    }
}
