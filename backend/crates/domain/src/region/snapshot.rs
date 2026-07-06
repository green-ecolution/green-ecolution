use uuid::Uuid;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RegionSnapshot {
    pub id: Uuid,
    pub name: String,
}
