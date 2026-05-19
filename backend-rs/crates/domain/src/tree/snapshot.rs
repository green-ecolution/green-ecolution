use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::shared::watering_status::WateringStatus;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct TreeSnapshot {
    pub id: Uuid,
    pub cluster_id: Option<Uuid>,
    pub sensor_id: Option<String>,
    pub planting_year: i32,
    pub species: String,
    pub tree_number: String,
    pub latitude: f64,
    pub longitude: f64,
    pub watering_status: WateringStatus,
    pub description: Option<String>,
    pub last_watered: Option<DateTime<Utc>>,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
}
