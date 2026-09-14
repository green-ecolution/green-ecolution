use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct PluginSnapshot {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub frontend_mode: String,
    pub frontend_target: Option<String>,
    pub organization_id: Uuid,
    pub permissions: Vec<String>,
    pub required_permissions: Vec<String>,
    pub enabled: bool,
    pub key_hash: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
}
