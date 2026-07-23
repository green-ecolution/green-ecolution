use chrono::{DateTime, Utc};

use crate::{Id, authorization::Permission, organization::Organization};

use super::{Role, RoleDescription, RoleName};

/// Flat read model returned by HTTP handlers. `created_at` derives from the
/// UUID v7 id (None for seeded/legacy non-v7 ids).
#[derive(Debug, Clone)]
pub struct RoleView {
    pub id: Id<Role>,
    pub organization_id: Option<Id<Organization>>,
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub permissions: Vec<Permission>,
    pub is_template: bool,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<&Role> for RoleView {
    fn from(value: &Role) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id(),
            name: value.name.clone(),
            description: value.description.clone(),
            permissions: value.permissions().iter().copied().collect(),
            is_template: value.is_template(),
            created_at: value.id.created_at(),
        }
    }
}
