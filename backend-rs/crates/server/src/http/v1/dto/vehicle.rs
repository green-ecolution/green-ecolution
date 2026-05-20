use serde::{Deserialize, Serialize};

use domain::{
    shared::{
        provenance::{Provenance, ProviderId},
        water_capacity::WaterCapacity,
    },
    vehicle::{NumberPlate, VehicleDimension, VehicleDraft, VehicleModel, VehicleView},
};

use super::{DrivingLicense, VehicleStatus, VehicleType};

/// Represents a watering vehicle used for urban green-space irrigation.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VehicleResponse {
    /// Unique identifier of the vehicle.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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

impl From<&VehicleView> for VehicleResponse {
    fn from(value: &VehicleView) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            number_plate: value.number_plate.clone(),
            description: value.description.clone().unwrap_or_default(),
            water_capacity: value.water_capacity,
            model: value.model.clone(),
            status: value.status.into(),
            vehicle_type: value.vehicle_type.into(),
            driving_license: value.driving_license.into(),
            height: value.height,
            width: value.width,
            length: value.length,
            weight: value.weight,
            archived_at: value.archived_at.map(|dt| dt.to_rfc3339()),
            provider: value.provider.clone(),
            additional_information: value.additional_info.clone(),
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
    pub description: Option<String>,
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
    pub description: Option<String>,
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

fn parse_provenance(
    provider: Option<String>,
    additional_information: Option<serde_json::Value>,
) -> Result<Provenance, crate::service::ServiceError> {
    let provider_id = provider
        .map(ProviderId::new)
        .transpose()
        .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
    Ok(Provenance::new(provider_id, additional_information))
}

impl VehicleCreateRequest {
    pub fn into_draft(self) -> Result<VehicleDraft, crate::service::ServiceError> {
        let number_plate = NumberPlate::new(self.number_plate)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let model = VehicleModel::new(self.model)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let water_capacity = WaterCapacity::new(self.water_capacity)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let dimension = VehicleDimension::new(self.height, self.width, self.length, self.weight)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let provenance = parse_provenance(self.provider, self.additional_information)?;

        Ok(VehicleDraft {
            number_plate,
            description: self.description,
            water_capacity,
            status: self.status.into(),
            vehicle_type: self.vehicle_type.into(),
            model,
            driving_license: self.driving_license.into(),
            dimension,
            provenance,
        })
    }
}

impl VehicleUpdateRequest {
    pub fn into_update(
        self,
    ) -> Result<domain::vehicle::VehicleUpdate, crate::service::ServiceError> {
        let number_plate = NumberPlate::new(self.number_plate)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let model = VehicleModel::new(self.model)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let water_capacity = WaterCapacity::new(self.water_capacity)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let dimension = VehicleDimension::new(self.height, self.width, self.length, self.weight)
            .map_err(|e| crate::service::ServiceError::InvalidInput(e.to_string()))?;
        let provenance = parse_provenance(self.provider, self.additional_information)?;

        Ok(domain::vehicle::VehicleUpdate {
            number_plate,
            description: self.description,
            water_capacity,
            status: self.status.into(),
            vehicle_type: self.vehicle_type.into(),
            model,
            driving_license: self.driving_license.into(),
            dimension,
            provenance,
        })
    }
}
