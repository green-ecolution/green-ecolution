use uuid::Uuid;

use crate::user::UserRole;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<UserRole>,
    pub raw_claims: serde_json::Value,
}
