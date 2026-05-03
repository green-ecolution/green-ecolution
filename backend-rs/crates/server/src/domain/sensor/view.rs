use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::sensor::{SensorStatus, data::SensorReadingView};

/// HTTP-side read model for a sensor.
///
/// Adds audit fields (`created_at`, `updated_at`) and embeds the
/// `latest_reading` for convenience. Uses primitive types rather than value
/// objects so HTTP serialisation doesn't need to unwrap newtypes.
#[derive(Debug, Clone)]
pub struct SensorView {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SensorStatus,
    pub latitude: f64,
    pub longitude: f64,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub latest_reading: Option<SensorReadingView>,
}
