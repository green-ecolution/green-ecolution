use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{
    shared::coordinates::Coordinate,
    watering_plan::{RefillPoint, WateringPlanStatus},
};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct WateringPlanSnapshot {
    pub id: Uuid,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub status: WateringPlanStatus,
    pub distance: Option<f64>,
    pub total_water_required: Option<f64>,
    pub cluster_ids: Vec<Uuid>,
    pub user_ids: Vec<Uuid>,
    pub transporter_id: Option<Uuid>,
    pub trailer_id: Option<Uuid>,
    pub cancellation_note: Option<String>,
    pub gpx_url: Option<Url>,
    pub refill_count: i32,
    pub duration: Duration,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub route_geometry: Option<Vec<Coordinate>>,
    pub refill_points: Vec<RefillPoint>,
}
