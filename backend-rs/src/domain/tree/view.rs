use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::shared::{distance::Distance, watering_status::WateringStatus};

#[derive(Debug, Clone)]
pub struct TreeView {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cluster_id: Option<i32>,
    pub sensor_id: Option<String>,
    pub planting_year: u32,
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

#[derive(Debug, Clone)]
pub struct TreeViewWithDistance {
    pub tree: TreeView,
    pub distance: Distance,
}
