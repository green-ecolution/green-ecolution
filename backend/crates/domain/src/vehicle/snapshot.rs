use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::vehicle::{DrivingLicense, VehicleStatus, VehicleType};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct VehicleSnapshot {
    pub id: Uuid,
    pub archived_at: Option<DateTime<Utc>>,
    pub number_plate: String,
    pub description: Option<String>,
    pub water_capacity: f64,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: String,
    pub driving_license: DrivingLicense,
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub organization_id: Uuid,
}
