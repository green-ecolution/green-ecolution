use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::vehicle::{DrivingLicense, VehicleStatus, VehicleType};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[derive(Debug, Clone)]
pub(crate) struct VehicleSnapshot {
    pub(crate) id: i32,
    pub(crate) archived_at: Option<DateTime<Utc>>,
    pub(crate) number_plate: String,
    pub(crate) description: Option<String>,
    pub(crate) water_capacity: f64,
    pub(crate) status: VehicleStatus,
    pub(crate) vehicle_type: VehicleType,
    pub(crate) model: String,
    pub(crate) driving_license: DrivingLicense,
    pub(crate) height: f64,
    pub(crate) width: f64,
    pub(crate) length: f64,
    pub(crate) weight: f64,
    pub(crate) provider: Option<String>,
    pub(crate) additional_info: Option<Value>,
}
