use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::sensor::{LorawanCredentials, SensorType};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct SensorSnapshot {
    pub id: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub sensor_type: SensorType,
    pub model_id: Uuid,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub lorawan: Option<LorawanCredentials>,
    pub organization_id: Uuid,
}
