use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::domain::{
    DomainError, Id, RepositoryError,
    shared::{
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        water_capacity::WaterCapacity,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "driving_license")]
pub enum DrivingLicense {
    B,
    BE,
    C,
    CE,
}

impl DrivingLicense {
    pub fn satisfies(&self, required: DrivingLicense) -> bool {
        *self == required
            || matches!(
                (self, required),
                (DrivingLicense::BE, DrivingLicense::B)
                    | (DrivingLicense::C, DrivingLicense::B)
                    | (
                        DrivingLicense::CE,
                        DrivingLicense::B | DrivingLicense::BE | DrivingLicense::C
                    )
            )
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "vehicle_status", rename_all = "snake_case")]
pub enum VehicleStatus {
    Active,
    Available,
    #[sqlx(rename = "not available")]
    NotAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "vehicle_type", rename_all = "snake_case")]
pub enum VehicleType {
    Transporter,
    Trailer,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub number_plate: String,
    pub description: Option<String>,
    pub water_capacity: WaterCapacity,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: String,
    pub driving_license: DrivingLicense,
    pub dimension: VehicleDimension,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Clone, Copy)]
pub struct VehicleDimension {
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
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

#[async_trait::async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn all(
        &self,
        query: VehicleQuery,
        pagination: Pagination,
    ) -> Result<Page<Vehicle>, RepositoryError>;
    async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, RepositoryError>;
    async fn by_plate(&self, plate: &str) -> Result<Vehicle, RepositoryError>;
    async fn create(&self, entity: VehicleCreate) -> Result<Vehicle, RepositoryError>;
    async fn update(
        &self,
        id: Id<Vehicle>,
        entity: VehicleUpdate,
    ) -> Result<Vehicle, RepositoryError>;
    async fn archive(&self, id: Id<Vehicle>) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Vehicle>) -> Result<(), RepositoryError>;
}
