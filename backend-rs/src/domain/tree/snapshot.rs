use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::shared::watering_status::WateringStatus;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[derive(Debug, Clone)]
pub(crate) struct TreeSnapshot {
    pub(crate) id: i32,
    pub(crate) cluster_id: Option<i32>,
    pub(crate) sensor_id: Option<String>,
    pub(crate) planting_year: i32,
    pub(crate) species: String,
    pub(crate) tree_number: String,
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) watering_status: WateringStatus,
    pub(crate) description: Option<String>,
    pub(crate) last_watered: Option<DateTime<Utc>>,
    pub(crate) provider: Option<String>,
    pub(crate) additional_info: Option<Value>,
}
