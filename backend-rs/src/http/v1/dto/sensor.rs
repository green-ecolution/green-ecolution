use serde::Serialize;

use crate::domain::sensor::{Sensor, SensorData, Watermark};

use super::SensorStatus;
use crate::http::v1::pagination::PaginationRepsonse;

#[derive(Debug, Serialize)]
pub struct WatermarkResponse {
    pub depth: i32,
    pub resistance: i32,
    pub centibar: i32,
}

impl From<&Watermark> for WatermarkResponse {
    fn from(value: &Watermark) -> Self {
        Self {
            depth: value.depth,
            resistance: value.resistance,
            centibar: value.centibar,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SensorDataResponse {
    pub created_at: String,
    pub updated_at: String,
    pub battery: f64,
    pub humidity: f64,
    pub temperature: f64,
    pub watermarks: Vec<WatermarkResponse>,
}

impl From<&SensorData> for SensorDataResponse {
    fn from(value: &SensorData) -> Self {
        Self {
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            battery: value.battery,
            humidity: value.humidity,
            temperature: value.temperature,
            watermarks: value.watermarks.iter().map(WatermarkResponse::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SensorResponse {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: SensorStatus,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_data: Option<SensorDataResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

impl From<&Sensor> for SensorResponse {
    fn from(value: &Sensor) -> Self {
        Self {
            id: value.id.value().to_string(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            status: value.status.map(Into::into).unwrap_or(SensorStatus::Unknown),
            latitude: value.coordinates.latitude(),
            longitude: value.coordinates.longitude(),
            latest_data: value.latest_data.as_ref().map(SensorDataResponse::from),
            provider: Some(value.provider_info.provider.clone()),
            additional_information: Some(value.provider_info.additional_info.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SensorListResponse {
    pub data: Vec<SensorResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationRepsonse>,
}

#[derive(Debug, Serialize)]
pub struct SensorDataListResponse {
    pub data: Vec<SensorDataResponse>,
}
