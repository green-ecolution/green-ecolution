use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{sensor::SensorId, uuid_v7_timestamp};

/// A single time-series measurement from a sensor.
///
/// `recorded_at` is the domain name for the event timestamp; it is derived
/// from the UUID v7 `id` (which encodes the original creation time).
#[derive(Debug, Clone, PartialEq)]
pub struct SensorReading {
    pub id: Uuid,
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
/// `created_at` is derived from the UUID v7 timestamp embedded in `id` to
/// preserve the existing API contract after the `sensor_data.created_at`
/// column was dropped in the UUID migration.
#[derive(Debug, Clone)]
pub struct SensorReadingView {
    pub id: Uuid,
    pub sensor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: Value,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct SensorReadingSnapshot {
    pub id: Uuid,
    pub sensor_id: String,
    pub data: Value,
}

impl SensorReading {
    #[doc(hidden)]
    pub fn reconstitute(snap: SensorReadingSnapshot) -> Self {
        let recorded_at = uuid_v7_timestamp(&snap.id)
            .expect("sensor_data.id is minted as uuid v7; non-v7 ids would be a schema bug");
        Self {
            id: snap.id,
            sensor_id: SensorId::reconstitute(snap.sensor_id),
            recorded_at,
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

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VolumetricReading {
    pub depth_cm: i32,
    pub moisture_percent: f64,
}
