//! Vehicle aggregate — watering trucks and trailers in the fleet.
//!
//! `archived_at` is part of the aggregate (it carries domain meaning — when a
//! vehicle was retired), unlike `created_at` / `updated_at` which are
//! infrastructure concerns dropped from the aggregate. The `archive` method is
//! idempotent: a second call after the vehicle is already archived is a no-op
//! and does not overwrite the original timestamp.
//!
//! [`VehicleView`] adds `created_at` / `updated_at` for HTTP responses.

pub mod error;
pub mod license;
pub mod repository;
pub mod snapshot;
pub mod view;

use chrono::{DateTime, Utc};

use crate::{
    Id,
    shared::{
        error::ValidationError,
        provenance::{Provenance, ProviderId},
        string_value::NonEmptyString,
        water_capacity::WaterCapacity,
    },
};

pub use error::VehicleError;
pub use license::DrivingLicense;
pub use repository::{VehicleReader, VehicleWriter};
#[allow(unused_imports)]
#[doc(hidden)]
pub use snapshot::VehicleSnapshot;
pub use view::VehicleView;

/// Operational status of a vehicle.
///
/// - `Active` — currently on a watering run.
/// - `Available` — ready to be assigned to a watering plan.
/// - `NotAvailable` — temporarily out of service (maintenance, etc.).
/// - `Unknown` — status not yet set or not determinable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "vehicle_status", rename_all = "snake_case")
)]
pub enum VehicleStatus {
    Active,
    Available,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "not available"))]
    NotAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "vehicle_type", rename_all = "snake_case")
)]
pub enum VehicleType {
    Transporter,
    Trailer,
}

/// Vehicle registration plate, 1–32 characters after trimming.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberPlate(NonEmptyString);

impl NumberPlate {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(
            value,
            "vehicle.number_plate",
            1,
            32,
        )?))
    }

    #[allow(dead_code)]
    pub fn reconstitute(value: String) -> Self {
        Self(NonEmptyString::reconstitute(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for NumberPlate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Vehicle make/model string, 1–128 characters after trimming.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VehicleModel(NonEmptyString);

impl VehicleModel {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(value, "vehicle.model", 1, 128)?))
    }

    #[allow(dead_code)]
    pub fn reconstitute(value: String) -> Self {
        Self(NonEmptyString::reconstitute(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for VehicleModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleDimension {
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
}

impl VehicleDimension {
    pub fn new(height: f64, width: f64, length: f64, weight: f64) -> Result<Self, ValidationError> {
        for (field, value) in [
            ("vehicle.height", height),
            ("vehicle.width", width),
            ("vehicle.length", length),
            ("vehicle.weight", weight),
        ] {
            if value < 0.0 {
                return Err(ValidationError::OutOfRange {
                    field,
                    min: 0.0,
                    max: f64::INFINITY,
                    got: value,
                });
            }
        }
        Ok(Self {
            height,
            width,
            length,
            weight,
        })
    }

    #[allow(dead_code)]
    pub fn reconstitute(height: f64, width: f64, length: f64, weight: f64) -> Self {
        Self {
            height,
            width,
            length,
            weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vehicle {
    pub id: Id<Vehicle>,
    pub number_plate: NumberPlate,
    pub description: Option<String>,
    pub water_capacity: WaterCapacity,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: VehicleModel,
    pub driving_license: DrivingLicense,
    pub dimension: VehicleDimension,

    archived_at: Option<DateTime<Utc>>,
    provenance: Provenance,
}

/// Input for creating a new [`Vehicle`].
#[derive(Debug, Clone)]
pub struct VehicleDraft {
    pub number_plate: NumberPlate,
    pub description: Option<String>,
    pub water_capacity: WaterCapacity,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: VehicleModel,
    pub driving_license: DrivingLicense,
    pub dimension: VehicleDimension,
    pub provenance: Provenance,
}

#[derive(Debug, Default, Clone)]
pub struct VehicleSearchQuery {
    pub vehicle_type: Option<VehicleType>,
    pub with_archived: bool,
    pub only_archived: bool,
    pub provider: Option<ProviderId>,
}

/// Replacement input for [`Vehicle`] updates.
#[derive(Debug, Clone)]
pub struct VehicleUpdate {
    pub number_plate: NumberPlate,
    pub description: Option<String>,
    pub water_capacity: WaterCapacity,
    pub status: VehicleStatus,
    pub vehicle_type: VehicleType,
    pub model: VehicleModel,
    pub driving_license: DrivingLicense,
    pub dimension: VehicleDimension,
    pub provenance: Provenance,
}

impl Vehicle {
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn reconstitute(snap: VehicleSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            number_plate: NumberPlate::reconstitute(snap.number_plate),
            description: snap.description,
            water_capacity: WaterCapacity::new(snap.water_capacity)
                .expect("DB water_capacity must be valid"),
            status: snap.status,
            vehicle_type: snap.vehicle_type,
            model: VehicleModel::reconstitute(snap.model),
            driving_license: snap.driving_license,
            dimension: VehicleDimension::reconstitute(
                snap.height,
                snap.width,
                snap.length,
                snap.weight,
            ),
            archived_at: snap.archived_at,
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
        }
    }

    pub fn archived_at(&self) -> Option<DateTime<Utc>> {
        self.archived_at
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn replace_details(&mut self, update: VehicleUpdate) {
        self.number_plate = update.number_plate;
        self.description = update.description;
        self.water_capacity = update.water_capacity;
        self.status = update.status;
        self.vehicle_type = update.vehicle_type;
        self.model = update.model;
        self.driving_license = update.driving_license;
        self.dimension = update.dimension;
        self.provenance = update.provenance;
    }

    /// Marks the vehicle as archived at `at`.
    ///
    /// Idempotent: if the vehicle is already archived the original timestamp is
    /// preserved and `at` is ignored.
    pub fn archive(&mut self, at: DateTime<Utc>) {
        if self.archived_at.is_some() {
            return;
        }
        self.archived_at = Some(at);
    }

    pub fn unarchive(&mut self) {
        self.archived_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_vehicle() -> Vehicle {
        Vehicle {
            id: Id::new(1),
            number_plate: NumberPlate::new("FL-AB-123").unwrap(),
            description: None,
            water_capacity: WaterCapacity::new(1000.0).unwrap(),
            status: VehicleStatus::Available,
            vehicle_type: VehicleType::Transporter,
            model: VehicleModel::new("Mercedes Sprinter").unwrap(),
            driving_license: DrivingLicense::B,
            dimension: VehicleDimension::new(2.0, 2.0, 5.0, 3500.0).unwrap(),
            archived_at: None,
            provenance: Provenance::default(),
        }
    }

    #[test]
    fn number_plate_rejects_empty() {
        assert_err!(NumberPlate::new(""));
    }

    #[test]
    fn number_plate_accepts_valid() {
        assert_ok!(NumberPlate::new("FL-AB-123"));
    }

    #[test]
    fn vehicle_model_rejects_empty() {
        assert_err!(VehicleModel::new(""));
    }

    #[test]
    fn dimension_rejects_negative() {
        assert_err!(VehicleDimension::new(-1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn dimension_accepts_zero() {
        assert_ok!(VehicleDimension::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn driving_license_satisfies_lower_classes() {
        assert!(DrivingLicense::CE.satisfies(DrivingLicense::B));
        assert!(DrivingLicense::CE.satisfies(DrivingLicense::C));
        assert!(!DrivingLicense::B.satisfies(DrivingLicense::C));
    }

    #[test]
    fn archive_sets_timestamp() {
        let mut v = fixed_vehicle();
        let ts = Utc::now();
        v.archive(ts);
        assert_eq!(v.archived_at(), Some(ts));
        assert!(v.is_archived());
    }

    #[test]
    fn archive_when_already_archived_is_noop() {
        let mut v = fixed_vehicle();
        let ts = Utc::now();
        v.archive(ts);
        let later = ts + chrono::Duration::seconds(60);
        v.archive(later);
        assert_eq!(v.archived_at(), Some(ts));
    }

    #[test]
    fn unarchive_clears_timestamp() {
        let mut v = fixed_vehicle();
        v.archive(Utc::now());
        v.unarchive();
        assert!(!v.is_archived());
    }

    #[test]
    fn replace_details_overwrites_all_mutable_fields() {
        let mut v = fixed_vehicle();
        let original_archived = v.archived_at();
        v.replace_details(VehicleUpdate {
            number_plate: NumberPlate::new("FL-XY-789").unwrap(),
            description: Some("neu".into()),
            water_capacity: WaterCapacity::new(2000.0).unwrap(),
            status: VehicleStatus::NotAvailable,
            vehicle_type: VehicleType::Trailer,
            model: VehicleModel::new("Schmitz Cargobull").unwrap(),
            driving_license: DrivingLicense::BE,
            dimension: VehicleDimension::new(2.5, 2.2, 6.0, 4000.0).unwrap(),
            provenance: Provenance::default(),
        });
        assert_eq!(v.number_plate.as_str(), "FL-XY-789");
        assert_eq!(v.description.as_deref(), Some("neu"));
        assert_eq!(v.water_capacity.liters(), 2000.0);
        assert_eq!(v.status, VehicleStatus::NotAvailable);
        assert_eq!(v.vehicle_type, VehicleType::Trailer);
        assert_eq!(v.model.as_str(), "Schmitz Cargobull");
        assert_eq!(v.driving_license, DrivingLicense::BE);
        assert_eq!(v.dimension.length, 6.0);
        assert_eq!(
            v.archived_at(),
            original_archived,
            "replace_details must not touch archived_at"
        );
    }
}
