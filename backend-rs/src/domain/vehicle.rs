use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::domain::{
    DomainError, Id,
    shared::{provider_info::ProviderInfo, water_capacity::WaterCapacity},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrivingLicense {
    B,
    BE,
    C,
    CE,
}

impl DrivingLicense {
    pub fn satisfies(&self, required: DrivingLicense) -> bool {
        match *self {
            DrivingLicense::BE => required == DrivingLicense::B,
            DrivingLicense::C => required == DrivingLicense::B,
            DrivingLicense::CE => {
                required == DrivingLicense::B
                    || required == DrivingLicense::BE
                    || required == DrivingLicense::C
            }
            _ => false,
        }
    }
}

impl FromStr for DrivingLicense {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(Self::B),
            "BE" => Ok(Self::BE),
            "C" => Ok(Self::C),
            "CE" => Ok(Self::CE),
            _ => Err(DomainError::InvalidDrivingLicense(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleStatus {
    Active,
    Available,
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Transporter,
    Trailer,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    archived_at: Option<DateTime<Utc>>,
    number_plate: String,
    description: Option<String>,
    water_capacity: WaterCapacity,
    vehicle_type: VehicleType,
    model: String,
    driving_license: DrivingLicense,
    dimension: VehicleDimension,
    provider_info: ProviderInfo,
}

impl Vehicle {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        archived_at: Option<DateTime<Utc>>,
        number_plate: String,
        description: Option<String>,
        water_capacity: WaterCapacity,
        vehicle_type: VehicleType,
        model: String,
        driving_license: DrivingLicense,
        dimension: VehicleDimension,
        provider_info: ProviderInfo,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            archived_at,
            number_plate,
            description,
            water_capacity,
            vehicle_type,
            model,
            driving_license,
            dimension,
            provider_info,
        }
    }

    pub fn id(&self) -> &Id<Self> {
        &self.id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    pub fn archived_at(&self) -> Option<DateTime<Utc>> {
        self.archived_at
    }
    pub fn number_plate(&self) -> &str {
        &self.number_plate
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn water_capacity(&self) -> &WaterCapacity {
        &self.water_capacity
    }
    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn driving_license(&self) -> DrivingLicense {
        self.driving_license
    }
    pub fn dimension(&self) -> &VehicleDimension {
        &self.dimension
    }
    pub fn provider_info(&self) -> &ProviderInfo {
        &self.provider_info
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VehicleDimension {
    height: f64,
    width: f64,
    length: f64,
    weight: f64,
}

impl VehicleDimension {
    pub fn new(height: f64, width: f64, length: f64, weight: f64) -> Self {
        Self {
            height,
            width,
            length,
            weight,
        }
    }

    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn length(&self) -> f64 {
        self.length
    }
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

#[derive(Debug)]
pub struct VehicleCreate {
    pub number_plate: String,
    pub description: String,
    pub water_capacity: WaterCapacity,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: String,
    pub driving_license: DrivingLicense,
    pub dimension: VehicleDimension,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct VehicleUpdate {
    pub number_plate: Option<String>,
    pub description: Option<String>,
    pub water_capacity: Option<WaterCapacity>,
    pub status: Option<VehicleStatus>,
    pub vehicle_type: Option<VehicleType>,
    pub model: Option<String>,
    pub driving_license: Option<DrivingLicense>,
    pub dimension: Option<VehicleDimension>,
    pub provider_info: Option<ProviderInfo>,
}

#[derive(Debug, Default)]
pub struct VehicleQuery {
    pub vehicle_type: Option<VehicleType>,
    pub with_archived: bool,
    pub only_archived: bool,
    pub provider: Option<String>,
}
