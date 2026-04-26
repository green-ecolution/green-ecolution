use serde::{Deserialize, Serialize};

use crate::domain::{
    DomainError,
    shared::{provider_info::ProviderInfo, water_capacity::WaterCapacity},
    vehicle::{Vehicle, VehicleCreate, VehicleDimension, VehicleUpdate},
};

use super::{DrivingLicense, VehicleStatus, VehicleType};

#[derive(Debug, Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
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

impl TryFrom<VehicleCreateRequest> for VehicleCreate {
    type Error = DomainError;

    fn try_from(req: VehicleCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            number_plate: req.number_plate,
            description: req.description,
            water_capacity: WaterCapacity::new(req.water_capacity)?,
            status: req.status.into(),
            vehicle_type: req.vehicle_type.into(),
            model: req.model,
            driving_license: req.driving_license.into(),
            dimension: VehicleDimension {
                height: req.height,
                width: req.width,
                length: req.length,
                weight: req.weight,
            },
            provider_info: ProviderInfo {
                provider: req.provider.unwrap_or_default(),
                additional_info: req.additional_information.unwrap_or_default(),
            },
        })
    }
}

impl TryFrom<VehicleUpdateRequest> for VehicleUpdate {
    type Error = DomainError;

    fn try_from(req: VehicleUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            number_plate: Some(req.number_plate),
            description: Some(req.description),
            water_capacity: Some(WaterCapacity::new(req.water_capacity)?),
            status: Some(req.status.into()),
            vehicle_type: Some(req.vehicle_type.into()),
            model: Some(req.model),
            driving_license: Some(req.driving_license.into()),
            dimension: Some(VehicleDimension {
                height: req.height,
                width: req.width,
                length: req.length,
                weight: req.weight,
            }),
            provider_info: Some(ProviderInfo {
                provider: req.provider.unwrap_or_default(),
                additional_info: req.additional_information.unwrap_or_default(),
            }),
        })
    }
}
