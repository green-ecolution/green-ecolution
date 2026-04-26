use serde::{Deserialize, Serialize};

use crate::domain::vehicle::Vehicle;

use super::{DrivingLicense, VehicleStatus, VehicleType};
use crate::http::v1::pagination::PaginationRepsonse;

#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub number_plate: String,
    pub description: String,
    pub water_capacity: f64,
    pub model: String,
    pub status: VehicleStatus,
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    pub driving_license: DrivingLicense,
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

impl From<&Vehicle> for VehicleResponse {
    fn from(value: &Vehicle) -> Self {
        Self {
            id: value.id.value(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            number_plate: value.number_plate.clone(),
            description: value.description.clone().unwrap_or_default(),
            water_capacity: value.water_capacity.liters(),
            model: value.model.clone(),
            status: value.status.into(),
            vehicle_type: value.vehicle_type.into(),
            driving_license: value.driving_license.into(),
            height: value.dimension.height,
            width: value.dimension.width,
            length: value.dimension.length,
            weight: value.dimension.weight,
            archived_at: value.archived_at.map(|dt| dt.to_rfc3339()),
            provider: Some(value.provider_info.provider.clone()),
            additional_information: Some(value.provider_info.additional_info.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VehicleListResponse {
    pub data: Vec<VehicleResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationRepsonse>,
}

#[derive(Debug, Deserialize)]
pub struct VehicleCreateRequest {
    pub number_plate: String,
    pub description: String,
    pub water_capacity: f64,
    pub model: String,
    pub status: VehicleStatus,
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    pub driving_license: DrivingLicense,
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct VehicleUpdateRequest {
    pub number_plate: String,
    pub description: String,
    pub water_capacity: f64,
    pub model: String,
    pub status: VehicleStatus,
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    pub driving_license: DrivingLicense,
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}
