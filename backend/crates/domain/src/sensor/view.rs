use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    sensor::{SensorStatus, SensorType, data::SensorReadingView},
    shared::{coordinates::Coordinate, provenance::ProviderId},
};

/// HTTP-side read model for a sensor.
///
/// Adds audit fields (`created_at`, `updated_at`) and embeds the
/// `latest_reading` for convenience. The sensor's own id is a LoRaWAN EUI
/// (String); only foreign-key targets use UUIDs.
#[derive(Debug, Clone)]
pub struct SensorView {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SensorStatus,
    pub sensor_type: SensorType,
    pub coordinate: Option<Coordinate>,
    pub linked_tree_id: Option<Uuid>,
    pub provider: Option<ProviderId>,
    pub additional_info: Option<Value>,
    pub model: SensorModelSummary,
    pub lorawan: Option<LorawanInfo>,
    pub latest_reading: Option<SensorReadingView>,
    pub organization_id: Uuid,
    /// Effective shares of the linked tree (own shares ∪ its cluster's
    /// shares); empty for an unlinked sensor.
    pub shared_with: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct SensorModelSummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LorawanInfo {
    pub serial_number: String,
    pub dev_eui: String,
    pub app_eui: String,
    pub at_pin: Option<String>,
    pub ota_pin: Option<String>,
    pub config: Option<serde_json::Value>,
}
