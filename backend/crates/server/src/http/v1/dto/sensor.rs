use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use domain::sensor_model::{SensorAbilityUnit, SensorModel};
use domain::shared::string_value::NonEmptyString;
use domain::{
    Id,
    organization::Organization,
    sensor::{
        LorawanCredentials, SensorDraft, SensorId, SensorType, SensorView,
        data::{SensorReadingView, SignalQuality},
        view::{LorawanInfo, SensorModelSummary},
    },
    shared::provenance::{Provenance, ProviderId},
};

use crate::service::{ServiceError, sensor_service::SensorService};

use super::SensorStatus;

/// Resolves a batch of raw sensor-id strings (e.g. from `TreeView::sensor_id`)
/// into a lookup map keyed by id. Strings that fail [`SensorId`] validation
/// are skipped silently - the caller already produced them, so an invalid
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

/// LoRaWAN radio quality of the strongest receiving gateway for a reading.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SignalResponse {
    /// Received signal strength indicator in dBm (higher/less-negative is better).
    #[schema(example = -104)]
    pub rssi_dbm: i32,
    /// Signal-to-noise ratio in dB.
    #[schema(example = 2.5)]
    pub snr_db: f32,
    /// Number of gateways that received this uplink.
    #[schema(example = 2)]
    pub gateway_count: u8,
}

impl From<SignalQuality> for SignalResponse {
    fn from(s: SignalQuality) -> Self {
        Self {
            rssi_dbm: s.rssi_dbm,
            snr_db: s.snr_db,
            gateway_count: s.gateway_count,
        }
    }
}

/// A single data payload received from a LoRaWAN sensor.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorDataResponse {
    /// Unique identifier of the reading.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,

    /// Timestamp when the data was first recorded (RFC 3339).
    #[schema(example = "2025-06-01T08:00:00+00:00")]
    pub created_at: String,

    /// Timestamp when the data was last modified (RFC 3339).
    #[schema(example = "2025-06-01T08:05:00+00:00")]
    pub updated_at: String,

    /// Raw sensor payload as a JSON object (structure depends on the sensor type).
    #[schema(value_type = Object, example = json!({"humidity": 42.5, "temperature": 18.3}))]
    pub data: serde_json::Value,

    /// LoRaWAN radio quality of the strongest gateway for this reading, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub signal: Option<SignalResponse>,
}

impl From<&SensorReadingView> for SensorDataResponse {
    fn from(value: &SensorReadingView) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            data: value.data.clone(),
            signal: value
                .data
                .get("signal")
                .and_then(|v| serde_json::from_value::<SignalQuality>(v.clone()).ok())
                .map(SignalResponse::from),
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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

/// LoRaWAN credentials that must never cross the HTTP boundary:
/// - `APPKEY`  - OTAA root key (full device impersonation)
/// - `APPSKEY` - application session key (decrypts uplinks)
/// - `NWKSKEY` - network session key (forges MAC frames)
/// - `PWORD`   - device AT-interface password
///
/// Comparison is case-insensitive because vendor exports normalise casing
/// inconsistently across firmware revisions.
const SENSITIVE_LORAWAN_CONFIG_KEYS: &[&str] = &["APPKEY", "APPSKEY", "NWKSKEY", "PWORD"];

fn redact_lorawan_config(config: &serde_json::Value) -> serde_json::Value {
    let serde_json::Value::Object(map) = config else {
        return config.clone();
    };
    serde_json::Value::Object(
        map.iter()
            .filter(|(k, _)| {
                !SENSITIVE_LORAWAN_CONFIG_KEYS
                    .iter()
                    .any(|sensitive| sensitive.eq_ignore_ascii_case(k))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

/// LoRaWAN connection details exposed publicly. The `config` map is filtered
/// via [`redact_lorawan_config`] to strip OTAA / session keys and the device
/// password before serialization.
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
            config: value.config.as_ref().map(redact_lorawan_config),
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub linked_tree_id: Option<uuid::Uuid>,

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

    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: String,
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
            organization_id: value.organization_id.to_string(),
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

/// Request body for `POST /sensors` - registers a prepared sensor unit.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSensorRequest {
    /// Sensor identifier (EUI), 1–64 characters after trimming.
    #[schema(example = "eui-a81758fffe0c3b52")]
    pub id: String,
    pub sensor_type: SensorTypeResponse,
    /// `SensorModel` id; must reference an existing model.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub model_id: uuid::Uuid,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}

impl CreateSensorRequest {
    pub fn into_draft(
        self,
        organization_id: Id<Organization>,
    ) -> Result<SensorDraft, ServiceError> {
        let sensor_type: SensorType = self.sensor_type.into();
        let lorawan = match (sensor_type, self.lorawan) {
            (SensorType::Lorawan, Some(l)) => parse_lorawan(l)?,
            (SensorType::Lorawan, None) => {
                return Err(ServiceError::InvalidInput(
                    "lorawan block required for sensor_type=lorawan".into(),
                ));
            }
        };

        Ok(SensorDraft {
            id: SensorId::new(self.id)?,
            sensor_type,
            model_id: Id::new(self.model_id),
            provenance: Provenance::new(
                self.provider.map(ProviderId::new).transpose()?,
                self.additional_information,
            ),
            lorawan,
            organization_id,
        })
    }
}

fn parse_lorawan(l: LorawanCredentialsRequest) -> Result<LorawanCredentials, ServiceError> {
    hex_field("dev_eui", &l.dev_eui, 16)?;
    hex_field("app_eui", &l.app_eui, 16)?;
    hex_field("app_key", &l.app_key, 32)?;

    Ok(LorawanCredentials {
        serial_number: NonEmptyString::new(l.serial_number, "sensor.lorawan.serial_number", 1, 64)?,
        dev_eui: NonEmptyString::new(l.dev_eui, "sensor.lorawan.dev_eui", 16, 16)?,
        app_eui: NonEmptyString::new(l.app_eui, "sensor.lorawan.app_eui", 16, 16)?,
        app_key: secrecy::SecretString::from(l.app_key),
        at_pin: l.at_pin,
        ota_pin: l.ota_pin,
        config: l.config,
    })
}

fn hex_field(label: &'static str, s: &str, len: usize) -> Result<(), ServiceError> {
    if s.len() != len || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ServiceError::InvalidInput(format!(
            "{label} must be {len} hex characters"
        )));
    }
    Ok(())
}

/// Request body for `POST /sensors/{sensor_id}/activate` - binds a prepared
/// sensor to a tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ActivateSensorRequest {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub tree_id: uuid::Uuid,
}

/// Request body for `PUT /sensors/{sensor_id}/tree` - links an activated
/// sensor to (or moves it to) the given tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetSensorTreeRequest {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub tree_id: uuid::Uuid,
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
    Volt,
}

impl From<SensorAbilityUnit> for SensorAbilityUnitDto {
    fn from(value: SensorAbilityUnit) -> Self {
        match value {
            SensorAbilityUnit::Percent => Self::Percent,
            SensorAbilityUnit::Centibar => Self::Centibar,
            SensorAbilityUnit::Ohm => Self::Ohm,
            SensorAbilityUnit::Celsius => Self::Celsius,
            SensorAbilityUnit::Volt => Self::Volt,
        }
    }
}

/// A single ability (e.g. soil tension at 60 cm) supported by a sensor model.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorModelAbilityResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = "soil_tension")]
    pub ability: String,
    pub unit: SensorAbilityUnitDto,
    #[schema(example = 60)]
    pub depth_cm: i32,
}

/// Full description of a supported sensor model and its abilities.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SensorModelResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redact_lorawan_config_strips_known_secret_keys() {
        let input = json!({
            "APPKEY": "leak",
            "APPSKEY": "leak",
            "NWKSKEY": "leak",
            "PWORD": "leak",
            "DEUI": "public",
            "TDC": "60000",
        });
        let out = redact_lorawan_config(&input);
        let obj = out.as_object().expect("object");
        assert!(!obj.contains_key("APPKEY"));
        assert!(!obj.contains_key("APPSKEY"));
        assert!(!obj.contains_key("NWKSKEY"));
        assert!(!obj.contains_key("PWORD"));
        assert_eq!(obj.get("DEUI").and_then(|v| v.as_str()), Some("public"));
        assert_eq!(obj.get("TDC").and_then(|v| v.as_str()), Some("60000"));
    }

    #[test]
    fn redact_lorawan_config_is_case_insensitive() {
        let input = json!({
            "appkey": "leak",
            "AppSKey": "leak",
            "PwOrD": "leak",
        });
        let out = redact_lorawan_config(&input);
        assert!(out.as_object().expect("object").is_empty());
    }

    #[test]
    fn redact_lorawan_config_passes_through_non_object_values() {
        let input = json!("not an object");
        assert_eq!(redact_lorawan_config(&input), input);
    }

    #[test]
    fn lorawan_info_response_redacts_secrets_via_from_impl() {
        let info = LorawanInfo {
            serial_number: "LA1".into(),
            dev_eui: "AA".into(),
            app_eui: "BB".into(),
            at_pin: Some("CC".into()),
            ota_pin: Some("DD".into()),
            config: Some(json!({
                "APPKEY": "leak",
                "DEUI": "public",
            })),
        };
        let dto = LorawanInfoResponse::from(&info);
        let cfg = dto.config.expect("config present");
        let obj = cfg.as_object().expect("object");
        assert!(!obj.contains_key("APPKEY"));
        assert!(obj.contains_key("DEUI"));
        // Top-level identifiers pass through unchanged.
        assert_eq!(dto.at_pin.as_deref(), Some("CC"));
        assert_eq!(dto.ota_pin.as_deref(), Some("DD"));
    }

    #[test]
    fn sensor_data_response_exposes_signal_from_data() {
        let view = SensorReadingView {
            id: uuid::Uuid::now_v7(),
            sensor_id: "eui-1".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            data: json!({
                "battery": 3.6,
                "signal": {"rssi_dbm": -104, "snr_db": 2.5, "gateway_count": 2}
            }),
        };
        let dto = SensorDataResponse::from(&view);
        let sig = dto.signal.expect("signal present");
        assert_eq!(sig.rssi_dbm, -104);
        assert!((sig.snr_db - 2.5).abs() < 1e-6);
        assert_eq!(sig.gateway_count, 2);
    }

    #[test]
    fn sensor_data_response_signal_absent_when_missing() {
        let view = SensorReadingView {
            id: uuid::Uuid::now_v7(),
            sensor_id: "eui-1".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            data: json!({"battery": 3.6}),
        };
        assert!(SensorDataResponse::from(&view).signal.is_none());
    }
}
