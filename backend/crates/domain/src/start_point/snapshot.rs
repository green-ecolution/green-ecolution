use uuid::Uuid;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct StartPointSnapshot {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub watering_point: bool,
    pub is_default: bool,
    pub organization_id: Uuid,
}
