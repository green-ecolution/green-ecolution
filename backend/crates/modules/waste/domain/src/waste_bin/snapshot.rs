use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Unvalidated field values, used only to rebuild the aggregate.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct WasteBinSnapshot {
    pub id: Uuid,
    pub number: String,
    pub latitude: f64,
    pub longitude: f64,
    pub volume_liters: f64,
    pub depth_cm: i32,
    pub organization_id: Uuid,
    pub provider: Option<String>,
    pub additional_informations: Option<Value>,
    pub fill_fraction: Option<f64>,
    pub fill_measured_at: Option<DateTime<Utc>>,
    pub last_emptied_at: Option<DateTime<Utc>>,
}
