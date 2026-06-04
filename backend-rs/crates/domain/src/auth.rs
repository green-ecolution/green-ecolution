//! Auth identity type used across the domain layer.
//!
//! After the login-flow rework (GECO-141) the backend is a pure JWT resource
//! server. This module now only carries [`AuthUser`] — the decoded identity
//! extracted from a validated JWT by the HTTP auth middleware.

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
