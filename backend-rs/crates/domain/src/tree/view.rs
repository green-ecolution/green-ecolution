use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::shared::{distance::Distance, watering_status::WateringStatus};

/// HTTP-side read model for a tree.
///
/// Uses primitive types rather than value objects; `created_at` is derived
/// from the UUID v7 `id` (no DB column).
#[derive(Debug, Clone)]
pub struct TreeView {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cluster_id: Option<Uuid>,
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

/// A [`TreeView`] annotated with the distance from the query point.
///
/// Returned by `TreeReader::view_nearest`.
#[derive(Debug, Clone)]
pub struct TreeViewWithDistance {
    pub tree: TreeView,
    pub distance: Distance,
}
