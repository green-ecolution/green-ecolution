//! MQTT payload schemas: legacy EcoDrizzler shape + generic per-reading shape.

use crate::sensor::data::Watermark;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EcoDrizzlerPayload {
    pub device: String,
    pub battery: f64,
    pub humidity: f64,
    pub temperature: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub watermarks: Vec<Watermark>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GenericReadingPayload {
    pub device: String,
    #[serde(default)]
    pub battery: Option<f64>,
    pub readings: Vec<PayloadReading>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PayloadReading {
    pub ability: String,
    pub depth: i32,
    pub value: f64,
}
