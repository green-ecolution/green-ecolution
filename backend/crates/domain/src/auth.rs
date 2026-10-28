use uuid::Uuid;

use crate::{Id, authorization::Caller, organization::Organization};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub raw_claims: serde_json::Value,
    pub organization_scope: Option<Id<Organization>>,
}

impl AuthUser {
    pub fn caller(&self) -> Caller {
        Caller {
            user_id: self.id,
            scope: self.organization_scope,
        }
    }
}
