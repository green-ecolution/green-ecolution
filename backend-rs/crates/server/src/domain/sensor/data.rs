use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;

use crate::domain::sensor::SensorId;

/// A single time-series measurement from a sensor.
///
/// `recorded_at` is the domain name for the event timestamp; the underlying
/// DB column is called `created_at`.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorReading {
    pub id: i32,
    pub sensor_id: SensorId,
    pub recorded_at: DateTime<Utc>,
    pub data: Value,
}

/// Input for recording a new sensor measurement.
#[derive(Debug, Clone)]
pub struct SensorReadingDraft {
    pub sensor_id: SensorId,
    pub data: Value,
}

/// HTTP-side read model for a sensor reading.
///
/// Exposes the timestamp as `created_at` (matching the DB column name and the
/// existing API contract) rather than the domain's `recorded_at`.
#[derive(Debug, Clone)]
pub struct SensorReadingView {
    pub id: i32,
    pub sensor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct SensorReadingSnapshot {
    pub(crate) id: i32,
    pub(crate) sensor_id: String,
    pub(crate) recorded_at: NaiveDateTime,
    pub(crate) data: Value,
}

impl SensorReading {
    #[allow(dead_code)]
    pub(crate) fn reconstitute(snap: SensorReadingSnapshot) -> Self {
        Self {
            id: snap.id,
            sensor_id: SensorId::reconstitute(snap.sensor_id),
            recorded_at: snap.recorded_at.and_utc(),
            data: snap.data,
        }
    }
}

/// Single Watermark soil-tension reading at a fixed depth in centimetres.
///
/// Centibar (kPa) is what the watering-status calibration tables consume;
/// resistance is recorded for raw-data archival.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Watermark {
    pub depth: i32,
    pub resistance: i32,
    pub centibar: i32,
}

/// Typed MQTT payload from a tree sensor (one uplink message).
///
/// Mirrors the Go backend's `MqttPayload`: device id, position, environmental
/// readings, and three watermark readings at depths 30/60/90 cm. Used both as
/// the input to [`crate::service::sensor_service::SensorService::handle_message`]
/// and as the JSON shape persisted in the `sensor_data` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MqttPayload {
    pub device: String,
    pub battery: f64,
    pub humidity: f64,
    pub temperature: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub watermarks: Vec<Watermark>,
}
