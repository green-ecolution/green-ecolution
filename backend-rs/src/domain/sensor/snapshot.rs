use serde_json::Value;

use crate::domain::sensor::SensorStatus;

#[derive(Debug, Clone)]
pub(crate) struct SensorSnapshot {
    pub(crate) id: String,
    pub(crate) status: SensorStatus,
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) provider: Option<String>,
    pub(crate) additional_info: Option<Value>,
}
