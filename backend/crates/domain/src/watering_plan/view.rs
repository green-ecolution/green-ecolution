use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::watering_plan::WateringPlanStatus;

/// HTTP-side read model for a watering plan.
///
/// `created_at` is derived from the UUID v7 `id`; `updated_at` is the DB column.
#[derive(Debug, Clone)]
pub struct WateringPlanView {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub refill_count: i32,
    pub duration: Duration,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub organization_id: Uuid,
}
