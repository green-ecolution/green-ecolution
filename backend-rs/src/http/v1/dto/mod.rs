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

use crate::domain::{
    cluster::SoilCondition as DomainSoilCondition,
    sensor::SensorStatus as DomainSensorStatus,
    shared::watering_status::WateringStatus as DomainWateringStatus,
    vehicle::{
        DrivingLicense as DomainDrivingLicense, VehicleStatus as DomainVehicleStatus,
        VehicleType as DomainVehicleType,
    },
    watering_plan::WateringPlanStatus as DomainWateringPlanStatus,
};

// -- Shared enums used across multiple DTOs --

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    #[serde(rename = "just watered")]
    JustWatered,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum DrivingLicense {
    B,
    BE,
    C,
    CE,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    Active,
    Available,
    #[serde(rename = "not available")]
    NotAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    Transporter,
    Trailer,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WateringPlanStatus {
    Planned,
    Active,
    Canceled,
    Finished,
    #[serde(rename = "not competed")]
    NotCompleted,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    Tbz,
    GreenEcolution,
    SmarteGrenzregion,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Available,
    Absent,
    Unknown,
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
            DomainVehicleType::Unknown => Self::Unknown,
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
