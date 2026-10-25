use std::collections::BTreeSet;

use uuid::Uuid;

use crate::capacity::CapacityKind;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct StartPointSnapshot {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub replenishes: BTreeSet<CapacityKind>,
    pub is_default: bool,
    pub organization_id: Uuid,
}
