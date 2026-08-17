use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Id, shared::address::Address};

use super::{Organization, OrganizationName};

/// Flat read model returned by HTTP handlers. `created_at` derives from the
/// UUID v7 id (None for seeded ids without a real timestamp). `member_count`
/// counts the directly assigned users, not the subtree.
#[derive(Debug, Clone)]
pub struct OrganizationView {
    pub id: Id<Organization>,
    pub parent_id: Option<Id<Organization>>,
    pub name: OrganizationName,
    pub address: Option<Address>,
    pub contact_person_id: Option<Uuid>,
    pub member_count: i64,
    pub created_at: Option<DateTime<Utc>>,
}

impl OrganizationView {
    pub fn new(org: &Organization, member_count: i64) -> Self {
        Self {
            id: org.id,
            parent_id: org.parent_id(),
            name: org.name.clone(),
            address: org.address().cloned(),
            contact_person_id: org.contact_person(),
            member_count,
            created_at: org.id.created_at(),
        }
    }
}
