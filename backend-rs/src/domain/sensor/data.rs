use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;

use crate::domain::sensor::SensorId;

#[derive(Debug, Clone, PartialEq)]
pub struct SensorReading {
    pub id: i32,
    pub sensor_id: SensorId,
    pub recorded_at: DateTime<Utc>,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct SensorReadingDraft {
    pub sensor_id: SensorId,
    pub data: Value,
}

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

#[derive(Debug, Clone)]
pub struct Watermark {
    pub depth: i32,
    pub resistance: i32,
    pub centibar: i32,
}
