use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Id, organization::Organization, settings::SettingKey};

/// One recorded change, for display only. `previous`/`next` are `None` where
/// the value was or becomes inherited, so going back to inheritance is a
/// visible change rather than a silent deletion.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingChangeEntry {
    pub organization_id: Id<Organization>,
    pub key: SettingKey,
    pub previous: Option<serde_json::Value>,
    pub next: Option<serde_json::Value>,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
}
