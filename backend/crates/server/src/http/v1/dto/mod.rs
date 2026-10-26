pub mod comment;
pub mod info;
pub mod module;
pub mod organization;
pub mod plan;
pub mod plugin;
pub mod region;
pub mod role;
pub mod routing;
pub mod sensor;
pub mod settings;
pub mod user;
pub mod vehicle;

use serde::{Deserialize, Serialize};

use crate::http::v1::pagination::PaginationResponse;
use domain::{
    sensor::SensorStatus as DomainSensorStatus,
    shared::{
        pagination::{Page, Pagination},
        sort::SortDirection,
    },
    vehicle::{
        DrivingLicense as DomainDrivingLicense, VehicleAvailability as DomainVehicleAvailability,
        VehicleStatus as DomainVehicleStatus, VehicleType as DomainVehicleType,
    },
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

    /// Adds the pre-filter total to the pagination block.
    pub fn with_total_unfiltered(mut self, total_unfiltered: u64) -> Self {
        self.pagination = self.pagination.with_total_unfiltered(total_unfiltered);
        self
    }

    /// Like [`Self::from_page_with`], but drops items the mapping declines.
    /// Use this when a data inconsistency in a single item (logged by the
    /// caller) must not take down the whole list response. `total` still
    /// reflects the unfiltered count.
    pub fn from_page_filter_map<D, F>(page: Page<D>, pagination: &Pagination, map_fn: F) -> Self
    where
        F: FnMut(&D) -> Option<T>,
    {
        Self {
            data: page.items.iter().filter_map(map_fn).collect(),
            pagination: PaginationResponse::new(page.total, pagination),
        }
    }
}

// -- Shared enums used across multiple DTOs --

/// Request body for transferring a resource's ownership to another
/// organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TransferRequest {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: uuid::Uuid,
}

#[derive(Debug, Clone, Copy, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrderParam {
    Asc,
    Desc,
}

impl From<SortOrderParam> for SortDirection {
    fn from(value: SortOrderParam) -> Self {
        match value {
            SortOrderParam::Asc => Self::Ascending,
            SortOrderParam::Desc => Self::Descending,
        }
    }
}

/// Connectivity status of a LoRaWAN sensor.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "online")]
pub enum SensorStatus {
    /// Sensor record exists but has not been activated yet.
    Prepared,
    /// Sensor is transmitting data within expected intervals.
    Online,
    /// Sensor has not transmitted data within the expected interval.
    Offline,
}

/// Whether a sensor's recent readings look trustworthy.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "ok")]
pub enum DataHealth {
    /// Recent readings passed the plausibility rules.
    Ok,
    /// Consecutive uplinks carried no plausible measurement.
    Suspect,
}

impl From<domain::sensor::DataHealth> for DataHealth {
    fn from(value: domain::sensor::DataHealth) -> Self {
        match value {
            domain::sensor::DataHealth::Ok => Self::Ok,
            domain::sensor::DataHealth::Suspect => Self::Suspect,
        }
    }
}

impl From<DataHealth> for domain::sensor::DataHealth {
    fn from(value: DataHealth) -> Self {
        match value {
            DataHealth::Ok => Self::Ok,
            DataHealth::Suspect => Self::Suspect,
        }
    }
}

/// European driving license category required to operate a vehicle.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

/// Whether a watering vehicle may be assigned at all. Set by hand; the only
/// part of a vehicle's state the system cannot observe for itself.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "available")]
pub enum VehicleAvailability {
    /// Vehicle may be assigned to a watering plan.
    Available,
    /// Vehicle is temporarily out of service (e.g. maintenance).
    #[serde(rename = "not_available", alias = "not available")]
    NotAvailable,
}

/// Operational status of a watering vehicle. Read-only: derived from the
/// vehicle's availability and the watering plans it is assigned to.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "available")]
pub enum VehicleStatus {
    /// Vehicle is currently in use on a watering plan.
    Active,
    /// Vehicle is available for assignment.
    Available,
    /// Vehicle is temporarily unavailable (e.g. maintenance).
    #[serde(rename = "not_available", alias = "not available")]
    NotAvailable,
}

/// Classification of a watering vehicle.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "transporter")]
pub enum VehicleType {
    /// Self-propelled vehicle with water tank.
    Transporter,
    /// Towed water tank attached to a transporter.
    Trailer,
}

/// Availability status of a user.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "available")]
pub enum UserStatus {
    /// User is available for watering plan assignments.
    Available,
    /// User is currently absent.
    Absent,
}

// -- From impls: DTO -> Domain --

impl From<VehicleAvailability> for DomainVehicleAvailability {
    fn from(value: VehicleAvailability) -> Self {
        match value {
            VehicleAvailability::Available => Self::Available,
            VehicleAvailability::NotAvailable => Self::NotAvailable,
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

impl From<VehicleStatus> for DomainVehicleStatus {
    fn from(value: VehicleStatus) -> Self {
        match value {
            VehicleStatus::Active => Self::Active,
            VehicleStatus::Available => Self::Available,
            VehicleStatus::NotAvailable => Self::NotAvailable,
        }
    }
}

// -- From impls: Domain -> DTO --

impl From<DomainSensorStatus> for SensorStatus {
    fn from(value: DomainSensorStatus) -> Self {
        match value {
            DomainSensorStatus::Prepared => Self::Prepared,
            DomainSensorStatus::Online => Self::Online,
            DomainSensorStatus::Offline => Self::Offline,
        }
    }
}

impl From<SensorStatus> for DomainSensorStatus {
    fn from(value: SensorStatus) -> Self {
        match value {
            SensorStatus::Prepared => Self::Prepared,
            SensorStatus::Online => Self::Online,
            SensorStatus::Offline => Self::Offline,
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
        }
    }
}

impl From<DomainVehicleAvailability> for VehicleAvailability {
    fn from(value: DomainVehicleAvailability) -> Self {
        match value {
            DomainVehicleAvailability::Available => Self::Available,
            DomainVehicleAvailability::NotAvailable => Self::NotAvailable,
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

#[cfg(test)]
mod wire_format_tests {
    use super::*;

    /// A value with a space cannot serve as an i18n key segment, and
    /// `not competed` was a typo carried over from the Go backend.
    #[test]
    fn no_status_value_contains_a_space() {
        let values = [
            serde_json::to_string(&VehicleStatus::NotAvailable).unwrap(),
            serde_json::to_string(&plan::PlanStatus::NotCompleted).unwrap(),
        ];
        for value in &values {
            assert!(
                !value.contains(' '),
                "a wire value must be usable as a key segment, got {value}"
            );
        }
    }

    #[test]
    fn statuses_serialize_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&VehicleStatus::NotAvailable).unwrap(),
            "\"not_available\""
        );
        assert_eq!(
            serde_json::to_string(&plan::PlanStatus::NotCompleted).unwrap(),
            "\"not_completed\""
        );
    }

    /// The rename would otherwise be a breaking change. Accepting the old
    /// spelling on input keeps every existing caller working.
    #[test]
    fn the_previous_spellings_are_still_accepted_on_input() {
        assert_eq!(
            serde_json::from_str::<VehicleStatus>("\"not available\"").unwrap(),
            VehicleStatus::NotAvailable
        );
        assert_eq!(
            serde_json::from_str::<plan::PlanStatus>("\"not competed\"").unwrap(),
            plan::PlanStatus::NotCompleted
        );
    }

    #[test]
    fn the_canonical_spellings_are_accepted_on_input() {
        assert_eq!(
            serde_json::from_str::<VehicleStatus>("\"not_available\"").unwrap(),
            VehicleStatus::NotAvailable
        );
        assert_eq!(
            serde_json::from_str::<plan::PlanStatus>("\"not_completed\"").unwrap(),
            plan::PlanStatus::NotCompleted
        );
    }
}
