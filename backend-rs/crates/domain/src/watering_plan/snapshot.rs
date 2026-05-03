use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use url::Url;

use crate::watering_plan::WateringPlanStatus;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct WateringPlanSnapshot {
    pub id: i32,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub status: WateringPlanStatus,
    pub distance: Option<f64>,
    pub total_water_required: Option<f64>,
    pub cluster_ids: Vec<i32>,
    pub transporter_id: Option<i32>,
    pub trailer_id: Option<i32>,
    pub cancellation_note: Option<String>,
    pub gpx_url: Option<Url>,
    pub refill_count: i32,
    pub duration: Duration,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
}
