use uuid::Uuid;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RoleSnapshot {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}
