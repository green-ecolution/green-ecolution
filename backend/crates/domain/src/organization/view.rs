use chrono::{DateTime, Utc};

use crate::Id;

use super::{Organization, OrganizationName};

/// Flat read model returned by HTTP handlers. `created_at` derives from the
/// UUID v7 id (None for seeded/legacy non-v7 ids).
#[derive(Debug, Clone)]
pub struct OrganizationView {
    pub id: Id<Organization>,
    pub parent_id: Option<Id<Organization>>,
    pub name: OrganizationName,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<&Organization> for OrganizationView {
    fn from(value: &Organization) -> Self {
        Self {
            id: value.id,
            parent_id: value.parent_id(),
            name: value.name.clone(),
            created_at: value.id.created_at(),
        }
    }
}
