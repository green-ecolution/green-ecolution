use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use domain::sensor::{
    SensorId, SensorType, SensorView,
    data::SensorReadingView,
    view::{LorawanInfo, SensorModelSummary},
};
use domain::sensor_model::{SensorAbilityUnit, SensorModel};

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

/// WGS-84 coordinate exposed in sensor responses (derived from the linked tree).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorCoordinate {
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
}

/// Summary view of the [`SensorModel`] this sensor belongs to.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorModelSummaryResponse {
    pub id: i32,
    pub name: String,
}

impl From<&SensorModelSummary> for SensorModelSummaryResponse {
    fn from(value: &SensorModelSummary) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
        }
    }
}

/// LoRaWAN connection details exposed publicly (omits `app_key`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LorawanInfoResponse {
    pub serial_number: String,
    pub dev_eui: String,
    pub app_eui: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ota_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>, nullable)]
    pub config: Option<serde_json::Value>,
}

impl From<&LorawanInfo> for LorawanInfoResponse {
    fn from(value: &LorawanInfo) -> Self {
        Self {
            serial_number: value.serial_number.clone(),
            dev_eui: value.dev_eui.clone(),
            app_eui: value.app_eui.clone(),
            at_pin: value.at_pin.clone(),
            ota_pin: value.ota_pin.clone(),
            config: value.config.clone(),
        }
    }
}

/// Sensor type (currently only LoRaWAN).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SensorTypeResponse {
    Lorawan,
}

impl From<domain::sensor::SensorType> for SensorTypeResponse {
    fn from(value: domain::sensor::SensorType) -> Self {
        match value {
            domain::sensor::SensorType::Lorawan => Self::Lorawan,
        }
    }
}

impl From<SensorTypeResponse> for SensorType {
    fn from(value: SensorTypeResponse) -> Self {
        match value {
            SensorTypeResponse::Lorawan => Self::Lorawan,
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

    /// Bus/protocol class of the sensor.
    pub sensor_type: SensorTypeResponse,

    /// Sensor model summary (id + display name).
    pub model: SensorModelSummaryResponse,

    /// WGS-84 coordinate derived from the linked tree (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub coordinate: Option<SensorCoordinate>,

    /// Database id of the linked tree, if the sensor is currently attached.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 42, nullable)]
    pub linked_tree_id: Option<i32>,

    /// LoRaWAN credentials (omits `app_key`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub lorawan: Option<LorawanInfoResponse>,

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
            sensor_type: value.sensor_type.into(),
            model: SensorModelSummaryResponse::from(&value.model),
            coordinate: value.coordinate.map(|c| SensorCoordinate {
                latitude: c.latitude(),
                longitude: c.longitude(),
            }),
            linked_tree_id: value.linked_tree_id,
            lorawan: value.lorawan.as_ref().map(LorawanInfoResponse::from),
            latest_data: value.latest_reading.as_ref().map(SensorDataResponse::from),
            provider: value.provider.as_ref().map(|p| p.as_str().to_owned()),
            additional_information: value.additional_info.clone(),
        }
    }
}

/// LoRaWAN credentials supplied when registering a new sensor. `app_key` is
/// write-only and never echoed back in responses.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LorawanCredentialsRequest {
    #[schema(example = "SN-2024-0001")]
    pub serial_number: String,
    #[schema(example = "a81758fffe0c3b52", min_length = 16, max_length = 16)]
    pub dev_eui: String,
    #[schema(example = "70b3d57ed0000000", min_length = 16, max_length = 16)]
    pub app_eui: String,
    #[schema(
        example = "00112233445566778899aabbccddeeff",
        min_length = 32,
        max_length = 32
    )]
    pub app_key: String,
    #[serde(default)]
    #[schema(nullable)]
    pub at_pin: Option<String>,
    #[serde(default)]
    #[schema(nullable)]
    pub ota_pin: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<Object>, nullable)]
    pub config: Option<serde_json::Value>,
}

/// Request body for `POST /sensors` — registers a prepared sensor unit.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSensorRequest {
    /// Sensor identifier (EUI), 1–64 characters after trimming.
    #[schema(example = "eui-a81758fffe0c3b52")]
    pub id: String,
    pub sensor_type: SensorTypeResponse,
    /// `SensorModel` id; must reference an existing model.
    #[schema(example = 1)]
    pub model_id: i32,
    #[serde(default)]
    #[schema(example = "tbz", nullable)]
    pub provider: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<Object>, nullable)]
    pub additional_information: Option<serde_json::Value>,
    /// Required when `sensor_type = lorawan`.
    #[serde(default)]
    #[schema(nullable)]
    pub lorawan: Option<LorawanCredentialsRequest>,
}

/// Request body for `POST /sensors/{sensor_id}/activate` — binds a prepared
/// sensor to a tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ActivateSensorRequest {
    #[schema(example = 42)]
    pub tree_id: i32,
}

/// Physical quantity reported by a sensor ability.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "centibar")]
pub enum SensorAbilityUnitDto {
    Percent,
    Centibar,
    Ohm,
    Celsius,
}

impl From<SensorAbilityUnit> for SensorAbilityUnitDto {
    fn from(value: SensorAbilityUnit) -> Self {
        match value {
            SensorAbilityUnit::Percent => Self::Percent,
            SensorAbilityUnit::Centibar => Self::Centibar,
            SensorAbilityUnit::Ohm => Self::Ohm,
            SensorAbilityUnit::Celsius => Self::Celsius,
        }
    }
}

/// A single ability (e.g. soil tension at 60 cm) supported by a sensor model.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorModelAbilityResponse {
    #[schema(example = 2)]
    pub id: i32,
    #[schema(example = "soil_tension")]
    pub ability: String,
    pub unit: SensorAbilityUnitDto,
    #[schema(example = 60)]
    pub depth_cm: i32,
}

/// Full description of a supported sensor model and its abilities.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorModelResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "EcoDrizzler")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub description: Option<String>,
    pub abilities: Vec<SensorModelAbilityResponse>,
}

impl From<&SensorModel> for SensorModelResponse {
    fn from(m: &SensorModel) -> Self {
        Self {
            id: m.id.value(),
            name: m.name.as_str().to_owned(),
            description: m.description.clone(),
            abilities: m
                .abilities
                .iter()
                .map(|a| SensorModelAbilityResponse {
                    id: a.id,
                    ability: a.ability.name.as_str().to_owned(),
                    unit: a.ability.unit.into(),
                    depth_cm: a.depth_cm,
                })
                .collect(),
        }
    }
}
