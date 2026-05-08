use std::collections::HashMap;

use serde::Serialize;

use domain::sensor::{SensorId, SensorView, data::SensorReadingView};

use crate::service::{ServiceError, sensor_service::SensorService};

use super::SensorStatus;

/// Resolves a batch of raw sensor-id strings (e.g. from `TreeView::sensor_id`)
/// into a lookup map keyed by id. Strings that fail [`SensorId`] validation
/// are skipped silently — the caller already produced them, so an invalid
/// value indicates dirty data, not a 400-worthy request error.
pub async fn resolve_sensors_by_str_ids<'a, I>(
    sensor_service: &SensorService,
    raw_ids: I,
) -> Result<HashMap<String, SensorView>, ServiceError>
where
    I: IntoIterator<Item = &'a str>,
{
    let ids: Vec<SensorId> = raw_ids
        .into_iter()
        .filter_map(|s| SensorId::new(s).ok())
        .collect();
    let sensors = sensor_service.view_by_ids(&ids).await?;
    Ok(sensors.into_iter().map(|s| (s.id.clone(), s)).collect())
}

/// A single data payload received from a LoRaWAN sensor.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorDataResponse {
    /// Timestamp when the data was first recorded (RFC 3339).
    #[schema(example = "2025-06-01T08:00:00+00:00")]
    pub created_at: String,

    /// Timestamp when the data was last modified (RFC 3339).
    #[schema(example = "2025-06-01T08:05:00+00:00")]
    pub updated_at: String,

    /// Raw sensor payload as a JSON object (structure depends on the sensor type).
    #[schema(value_type = Object, example = json!({"humidity": 42.5, "temperature": 18.3}))]
    pub data: serde_json::Value,
}

impl From<&SensorReadingView> for SensorDataResponse {
    fn from(value: &SensorReadingView) -> Self {
        Self {
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            data: value.data.clone(),
        }
    }
}

/// A LoRaWAN sensor used for soil moisture monitoring.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorResponse {
    /// Unique sensor identifier (EUI).
    #[schema(example = "eui-a81758fffe0c3b52")]
    pub id: String,

    /// Timestamp when the sensor was registered (RFC 3339).
    #[schema(example = "2024-01-15T10:30:00+00:00")]
    pub created_at: String,

    /// Timestamp when the sensor record was last updated (RFC 3339).
    #[schema(example = "2025-06-01T08:05:00+00:00")]
    pub updated_at: String,

    /// Current connectivity status of the sensor.
    pub status: SensorStatus,

    /// Latitude of the sensor location (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,

    /// Longitude of the sensor location (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,

    /// Most recent data payload from the sensor, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub latest_data: Option<SensorDataResponse>,

    /// Name of the data provider or integration (e.g. "ttn", "chirpstack").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "ttn", nullable)]
    pub provider: Option<String>,

    /// Provider-specific metadata as a JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>, nullable, example = json!({"app_id": "green-ecolution"}))]
    pub additional_information: Option<serde_json::Value>,
}

impl From<&SensorView> for SensorResponse {
    fn from(value: &SensorView) -> Self {
        Self {
            id: value.id.clone(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            status: value.status.into(),
            latitude: value.latitude,
            longitude: value.longitude,
            latest_data: value.latest_reading.as_ref().map(SensorDataResponse::from),
            provider: value.provider.clone(),
            additional_information: value.additional_info.clone(),
        }
    }
}
