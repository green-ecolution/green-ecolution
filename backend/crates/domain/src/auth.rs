use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub raw_claims: serde_json::Value,
}
