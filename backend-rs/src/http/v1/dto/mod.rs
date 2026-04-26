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
        shared::{pagination::Page, watering_status::WateringStatus as DomainWateringStatus},
        vehicle::{
            DrivingLicense as DomainDrivingLicense, VehicleStatus as DomainVehicleStatus,
            VehicleType as DomainVehicleType,
        },
        watering_plan::WateringPlanStatus as DomainWateringPlanStatus,
    },
    http::v1::pagination::PaginationRepsonse,
};

#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationRepsonse,
}

impl<T: Serialize> ListResponse<T> {
    pub fn from_page<D>(page: Page<D>, current_page: u64, per_page: u64) -> Self
    where
        T: for<'a> From<&'a D>,
    {
        let total_pages = (page.total + per_page - 1) / per_page;
        Self {
            data: page.items.iter().map(T::from).collect(),
            pagination: PaginationRepsonse::new(page.total, current_page, total_pages),
        }
    }

    pub fn from_page_with<D>(
        page: Page<D>,
        current_page: u64,
        per_page: u64,
        map_fn: impl Fn(&D) -> T,
    ) -> Self {
        let total_pages = (page.total + per_page - 1) / per_page;
        Self {
            data: page.items.iter().map(map_fn).collect(),
            pagination: PaginationRepsonse::new(page.total, current_page, total_pages),
        }
    }
}

// -- Shared enums used across multiple DTOs --

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    #[serde(rename = "just watered")]
    JustWatered,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum DrivingLicense {
    B,
    BE,
    C,
    CE,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    Active,
    Available,
    #[serde(rename = "not available")]
    NotAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    Transporter,
    Trailer,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    Tbz,
    GreenEcolution,
    SmarteGrenzregion,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Available,
    Absent,
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
            VehicleType::Unknown => Self::Unknown,
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
