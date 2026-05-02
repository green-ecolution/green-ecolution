pub mod cluster;
pub mod evaluation;
pub mod info;
pub mod plugin;
pub mod region;
pub mod sensor;
pub mod tree;
pub mod user;
pub mod vehicle;
pub mod watering_plan;

use serde::Serialize;

use crate::{
    domain::{
        cluster::SoilCondition as DomainSoilCondition,
        sensor::SensorStatus as DomainSensorStatus,
        shared::{
            pagination::{Page, Pagination},
            watering_status::WateringStatus as DomainWateringStatus,
        },
        vehicle::{
            DrivingLicense as DomainDrivingLicense, VehicleStatus as DomainVehicleStatus,
            VehicleType as DomainVehicleType,
        },
        watering_plan::WateringPlanStatus as DomainWateringPlanStatus,
    },
    http::v1::pagination::PaginationResponse,
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListResponse<T: Serialize + utoipa::ToSchema> {
    pub data: Vec<T>,
    pub pagination: PaginationResponse,
}

impl<T: Serialize + utoipa::ToSchema> ListResponse<T> {
    /// Build a `ListResponse` from a domain `Page` using the standard `T: From<&D>` conversion.
    pub fn from_page<D>(page: Page<D>, pagination: &Pagination) -> Self
    where
        T: for<'a> From<&'a D>,
    {
        Self {
            data: page.items.iter().map(T::from).collect(),
            pagination: PaginationResponse::new(page.total, pagination),
        }
    }

    /// Build a `ListResponse` from a domain `Page` with a custom mapping closure.
    /// Use this when the DTO needs additional context (e.g. a sensor map) beyond the domain item.
    pub fn from_page_with<D, F>(page: Page<D>, pagination: &Pagination, map_fn: F) -> Self
    where
        F: FnMut(&D) -> T,
    {
        Self {
            data: page.items.iter().map(map_fn).collect(),
            pagination: PaginationResponse::new(page.total, pagination),
        }
    }
}

// -- Shared enums used across multiple DTOs --

/// Current watering status of a tree or tree cluster.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "good")]
pub enum WateringStatus {
    /// Soil moisture is sufficient — no watering needed.
    Good,
    /// Soil moisture is below optimal — watering recommended soon.
    Moderate,
    /// Soil moisture is critically low — immediate watering required.
    Bad,
    /// Recently watered — status will update after next sensor reading.
    #[serde(rename = "just watered")]
    JustWatered,
    /// Status could not be determined (e.g. no sensor data available).
    Unknown,
}

/// Soil condition classification of a tree cluster site.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "sandig")]
pub enum SoilCondition {
    /// Silt-based soil with moderate water retention.
    Schluffig,
    /// Sandy soil with low water retention.
    Sandig,
    /// Loamy soil with good water retention.
    Lehmig,
    /// Clay-heavy soil with high water retention.
    Tonig,
    /// Soil condition has not been assessed.
    Unknown,
}

/// Connectivity status of a LoRaWAN sensor.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "online")]
pub enum SensorStatus {
    /// Sensor is transmitting data within expected intervals.
    Online,
    /// Sensor has not transmitted data within the expected interval.
    Offline,
    /// Sensor connectivity status could not be determined.
    Unknown,
}

/// European driving license category required to operate a vehicle.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(example = "BE")]
pub enum DrivingLicense {
    /// Standard car license (up to 3.5t).
    B,
    /// Car license with trailer (up to 3.5t + trailer).
    BE,
    /// Truck license (over 3.5t).
    C,
    /// Truck license with trailer.
    CE,
}

/// Operational status of a watering vehicle.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "available")]
pub enum VehicleStatus {
    /// Vehicle is currently in use on a watering plan.
    Active,
    /// Vehicle is available for assignment.
    Available,
    /// Vehicle is temporarily unavailable (e.g. maintenance).
    #[serde(rename = "not available")]
    NotAvailable,
    /// Vehicle status could not be determined.
    Unknown,
}

/// Classification of a watering vehicle.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "transporter")]
pub enum VehicleType {
    /// Self-propelled vehicle with water tank.
    Transporter,
    /// Towed water tank attached to a transporter.
    Trailer,
}

/// Lifecycle status of a watering plan.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "planned")]
pub enum WateringPlanStatus {
    /// Plan has been created but not yet started.
    Planned,
    /// Plan is currently being executed.
    Active,
    /// Plan was canceled before completion.
    Canceled,
    /// Plan was completed successfully.
    Finished,
    /// Plan was started but could not be completed.
    #[serde(rename = "not competed")]
    NotCompleted,
    /// Plan status could not be determined.
    Unknown,
}

/// Role assigned to a user within the system.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
#[schema(example = "tbz")]
pub enum UserRole {
    /// Technisches Betriebszentrum — primary operator role.
    Tbz,
    /// Green Ecolution project team member.
    GreenEcolution,
    /// Smarte Grenzregion project team member.
    SmarteGrenzregion,
    /// Role could not be determined.
    Unknown,
}

/// Availability status of a user.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "available")]
pub enum UserStatus {
    /// User is available for watering plan assignments.
    Available,
    /// User is currently absent.
    Absent,
    /// Availability could not be determined.
    Unknown,
}

// -- From impls: DTO -> Domain --

impl From<SoilCondition> for DomainSoilCondition {
    fn from(value: SoilCondition) -> Self {
        match value {
            SoilCondition::Schluffig => Self::Schluffig,
            SoilCondition::Sandig => Self::Sandig,
            SoilCondition::Lehmig => Self::Lehmig,
            SoilCondition::Tonig => Self::Tonig,
            SoilCondition::Unknown => Self::Unknown,
        }
    }
}

impl From<VehicleStatus> for DomainVehicleStatus {
    fn from(value: VehicleStatus) -> Self {
        match value {
            VehicleStatus::Active => Self::Active,
            VehicleStatus::Available => Self::Available,
            VehicleStatus::NotAvailable => Self::NotAvailable,
            VehicleStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<VehicleType> for DomainVehicleType {
    fn from(value: VehicleType) -> Self {
        match value {
            VehicleType::Transporter => Self::Transporter,
            VehicleType::Trailer => Self::Trailer,
        }
    }
}

impl From<DrivingLicense> for DomainDrivingLicense {
    fn from(value: DrivingLicense) -> Self {
        match value {
            DrivingLicense::B => Self::B,
            DrivingLicense::BE => Self::BE,
            DrivingLicense::C => Self::C,
            DrivingLicense::CE => Self::CE,
        }
    }
}

impl From<WateringPlanStatus> for DomainWateringPlanStatus {
    fn from(value: WateringPlanStatus) -> Self {
        match value {
            WateringPlanStatus::Planned => Self::Planned,
            WateringPlanStatus::Active => Self::Active,
            WateringPlanStatus::Canceled => Self::Canceled,
            WateringPlanStatus::Finished => Self::Finished,
            WateringPlanStatus::NotCompleted => Self::NotCompleted,
            WateringPlanStatus::Unknown => Self::Unknown,
        }
    }
}

// -- From impls: Domain -> DTO --

impl From<DomainWateringStatus> for WateringStatus {
    fn from(value: DomainWateringStatus) -> Self {
        match value {
            DomainWateringStatus::Good => Self::Good,
            DomainWateringStatus::Moderate => Self::Moderate,
            DomainWateringStatus::Bad => Self::Bad,
            DomainWateringStatus::JustWatered => Self::JustWatered,
            DomainWateringStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainSoilCondition> for SoilCondition {
    fn from(value: DomainSoilCondition) -> Self {
        match value {
            DomainSoilCondition::Schluffig => Self::Schluffig,
            DomainSoilCondition::Sandig => Self::Sandig,
            DomainSoilCondition::Lehmig => Self::Lehmig,
            DomainSoilCondition::Tonig => Self::Tonig,
            DomainSoilCondition::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainSensorStatus> for SensorStatus {
    fn from(value: DomainSensorStatus) -> Self {
        match value {
            DomainSensorStatus::Online => Self::Online,
            DomainSensorStatus::Offline => Self::Offline,
            DomainSensorStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainDrivingLicense> for DrivingLicense {
    fn from(value: DomainDrivingLicense) -> Self {
        match value {
            DomainDrivingLicense::B => Self::B,
            DomainDrivingLicense::BE => Self::BE,
            DomainDrivingLicense::C => Self::C,
            DomainDrivingLicense::CE => Self::CE,
        }
    }
}

impl From<DomainVehicleStatus> for VehicleStatus {
    fn from(value: DomainVehicleStatus) -> Self {
        match value {
            DomainVehicleStatus::Active => Self::Active,
            DomainVehicleStatus::Available => Self::Available,
            DomainVehicleStatus::NotAvailable => Self::NotAvailable,
            DomainVehicleStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainVehicleType> for VehicleType {
    fn from(value: DomainVehicleType) -> Self {
        match value {
            DomainVehicleType::Transporter => Self::Transporter,
            DomainVehicleType::Trailer => Self::Trailer,
        }
    }
}

impl From<DomainWateringPlanStatus> for WateringPlanStatus {
    fn from(value: DomainWateringPlanStatus) -> Self {
        match value {
            DomainWateringPlanStatus::Planned => Self::Planned,
            DomainWateringPlanStatus::Active => Self::Active,
            DomainWateringPlanStatus::Canceled => Self::Canceled,
            DomainWateringPlanStatus::Finished => Self::Finished,
            DomainWateringPlanStatus::NotCompleted => Self::NotCompleted,
            DomainWateringPlanStatus::Unknown => Self::Unknown,
        }
    }
}
