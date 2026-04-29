use serde::{Deserialize, Serialize};

use crate::domain::{
    DomainError,
    shared::{provider_info::ProviderInfo, water_capacity::WaterCapacity},
    vehicle::{Vehicle, VehicleCreate, VehicleDimension, VehicleUpdate},
};

use super::{DrivingLicense, VehicleStatus, VehicleType};

/// Represents a watering vehicle used for urban green-space irrigation.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VehicleResponse {
    /// Unique identifier of the vehicle.
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    /// Timestamp when the vehicle was created (RFC 3339).
    #[schema(example = "2025-01-15T08:30:00Z")]
    pub created_at: String,
    /// Timestamp when the vehicle was last updated (RFC 3339).
    #[schema(example = "2025-03-20T14:12:00Z")]
    pub updated_at: String,
    /// Official license plate number of the vehicle.
    #[schema(example = "FL-GE 123")]
    pub number_plate: String,
    /// Human-readable description of the vehicle.
    #[schema(example = "Großes Bewässerungsfahrzeug für den Innenstadtbereich")]
    pub description: String,
    /// Water tank capacity in liters.
    #[schema(example = 8000.0, minimum = 0.0)]
    pub water_capacity: f64,
    /// Manufacturer and model name.
    #[schema(example = "MAN TGE 3.180")]
    pub model: String,
    /// Current operational status of the vehicle.
    pub status: VehicleStatus,
    /// Type/category of the vehicle.
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    /// Required driving license class to operate this vehicle.
    pub driving_license: DrivingLicense,
    /// Vehicle height in meters.
    #[schema(example = 3.2, minimum = 0.0)]
    pub height: f64,
    /// Vehicle width in meters.
    #[schema(example = 2.5, minimum = 0.0)]
    pub width: f64,
    /// Vehicle length in meters.
    #[schema(example = 6.8, minimum = 0.0)]
    pub length: f64,
    /// Vehicle weight in kilograms.
    #[schema(example = 7500.0, minimum = 0.0)]
    pub weight: f64,
    /// Timestamp when the vehicle was archived, if applicable (RFC 3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2025-06-01T00:00:00Z", nullable)]
    pub archived_at: Option<String>,
    /// Name of the external data provider that supplied this vehicle record.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
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
            provider: value.provider_info.provider.clone(),
            additional_information: value.provider_info.additional_info.clone(),
        }
    }
}

/// Request body for creating a new watering vehicle.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct VehicleCreateRequest {
    /// Official license plate number of the vehicle.
    #[schema(example = "FL-GE 123")]
    pub number_plate: String,
    /// Human-readable description of the vehicle.
    #[schema(example = "Großes Bewässerungsfahrzeug für den Innenstadtbereich")]
    pub description: String,
    /// Water tank capacity in liters.
    #[schema(example = 8000.0, minimum = 0.0)]
    pub water_capacity: f64,
    /// Manufacturer and model name.
    #[schema(example = "MAN TGE 3.180")]
    pub model: String,
    /// Current operational status of the vehicle.
    pub status: VehicleStatus,
    /// Type/category of the vehicle.
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    /// Required driving license class to operate this vehicle.
    pub driving_license: DrivingLicense,
    /// Vehicle height in meters.
    #[schema(example = 3.2, minimum = 0.0)]
    pub height: f64,
    /// Vehicle width in meters.
    #[schema(example = 2.5, minimum = 0.0)]
    pub width: f64,
    /// Vehicle length in meters.
    #[schema(example = 6.8, minimum = 0.0)]
    pub length: f64,
    /// Vehicle weight in kilograms.
    #[schema(example = 7500.0, minimum = 0.0)]
    pub weight: f64,
    /// Name of the external data provider that supplied this vehicle record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for updating an existing watering vehicle.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct VehicleUpdateRequest {
    /// Official license plate number of the vehicle.
    #[schema(example = "FL-GE 123")]
    pub number_plate: String,
    /// Human-readable description of the vehicle.
    #[schema(example = "Großes Bewässerungsfahrzeug für den Innenstadtbereich")]
    pub description: String,
    /// Water tank capacity in liters.
    #[schema(example = 8000.0, minimum = 0.0)]
    pub water_capacity: f64,
    /// Manufacturer and model name.
    #[schema(example = "MAN TGE 3.180")]
    pub model: String,
    /// Current operational status of the vehicle.
    pub status: VehicleStatus,
    /// Type/category of the vehicle.
    #[serde(rename = "type")]
    pub vehicle_type: VehicleType,
    /// Required driving license class to operate this vehicle.
    pub driving_license: DrivingLicense,
    /// Vehicle height in meters.
    #[schema(example = 3.2, minimum = 0.0)]
    pub height: f64,
    /// Vehicle width in meters.
    #[schema(example = 2.5, minimum = 0.0)]
    pub width: f64,
    /// Vehicle length in meters.
    #[schema(example = 6.8, minimum = 0.0)]
    pub length: f64,
    /// Vehicle weight in kilograms.
    #[schema(example = 7500.0, minimum = 0.0)]
    pub weight: f64,
    /// Name of the external data provider that supplied this vehicle record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
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
                provider: req.provider,
                additional_info: req.additional_information,
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
                provider: req.provider,
                additional_info: req.additional_information,
            }),
        })
    }
}
