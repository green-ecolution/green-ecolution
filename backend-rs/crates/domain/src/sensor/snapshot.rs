use serde_json::Value;

use crate::sensor::{LorawanCredentials, SensorStatus, SensorType};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct SensorSnapshot {
    pub id: String,
    pub status: SensorStatus,
    pub sensor_type: SensorType,
    pub model_id: i32,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub lorawan: Option<LorawanCredentials>,
}
