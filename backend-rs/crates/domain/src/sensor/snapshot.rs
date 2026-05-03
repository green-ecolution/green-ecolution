use serde_json::Value;

use crate::sensor::SensorStatus;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct SensorSnapshot {
    pub id: String,
    pub status: SensorStatus,
    pub latitude: f64,
    pub longitude: f64,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
}
