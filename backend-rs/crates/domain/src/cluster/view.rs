use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{cluster::SoilCondition, shared::watering_status::WateringStatus};

/// HTTP-side read model for a tree cluster.
///
/// `created_at` is derived from the UUID v7 `id`; `updated_at` is the DB column.
#[derive(Debug, Clone)]
pub struct TreeClusterView {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub address: String,
    pub description: String,
    pub watering_status: WateringStatus,
    pub last_watered: Option<DateTime<Utc>>,
    pub moisture_level: f64,
    pub region_id: Option<Uuid>,
    pub archived: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub soil_condition: Option<SoilCondition>,
    pub tree_ids: Vec<Uuid>,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
}
