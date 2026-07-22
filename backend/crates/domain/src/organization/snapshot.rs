use uuid::Uuid;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OrganizationSnapshot {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
}
