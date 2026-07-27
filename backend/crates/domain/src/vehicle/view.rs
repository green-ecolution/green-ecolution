use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::vehicle::{DrivingLicense, VehicleStatus, VehicleType};

/// HTTP-side read model for a vehicle.
///
/// `created_at` is derived from the UUID v7 `id`; `updated_at` is the DB column.
#[derive(Debug, Clone)]
pub struct VehicleView {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
