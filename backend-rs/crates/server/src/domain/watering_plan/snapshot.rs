use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use url::Url;

use crate::domain::watering_plan::WateringPlanStatus;

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[derive(Debug, Clone)]
pub(crate) struct WateringPlanSnapshot {
    pub(crate) id: i32,
    pub(crate) date: DateTime<Utc>,
    pub(crate) description: Option<String>,
    pub(crate) status: WateringPlanStatus,
    pub(crate) distance: Option<f64>,
    pub(crate) total_water_required: Option<f64>,
    pub(crate) cluster_ids: Vec<i32>,
    pub(crate) transporter_id: Option<i32>,
    pub(crate) trailer_id: Option<i32>,
    pub(crate) cancellation_note: Option<String>,
    pub(crate) gpx_url: Option<Url>,
    pub(crate) refill_count: i32,
    pub(crate) duration: Duration,
    pub(crate) provider: Option<String>,
    pub(crate) additional_info: Option<Value>,
}
