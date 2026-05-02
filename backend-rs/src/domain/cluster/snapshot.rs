use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::{cluster::SoilCondition, shared::watering_status::WateringStatus};

#[derive(Debug, Clone)]
pub(crate) struct TreeClusterSnapshot {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) description: String,
    pub(crate) watering_status: WateringStatus,
    pub(crate) last_watered: Option<DateTime<Utc>>,
    pub(crate) moisture_level: f64,
    pub(crate) region_id: Option<i32>,
    pub(crate) archived: bool,
    pub(crate) latitude: Option<f64>,
    pub(crate) longitude: Option<f64>,
    pub(crate) soil_condition: Option<SoilCondition>,
    pub(crate) tree_ids: Vec<i32>,
    pub(crate) provider: Option<String>,
    pub(crate) additional_info: Option<Value>,
}
