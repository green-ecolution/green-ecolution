pub mod error;
pub mod license;
pub mod repository;
pub mod snapshot;
pub mod view;

use chrono::{DateTime, Utc};

use crate::domain::{
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
pub(crate) use snapshot::VehicleSnapshot;
pub use view::VehicleView;

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
}

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
    pub(crate) fn reconstitute(value: String) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VehicleModel(NonEmptyString);

impl VehicleModel {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(value, "vehicle.model", 1, 128)?))
    }

    #[allow(dead_code)]
    pub(crate) fn reconstitute(value: String) -> Self {
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
    pub(crate) fn reconstitute(height: f64, width: f64, length: f64, weight: f64) -> Self {
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

impl Vehicle {
    #[allow(dead_code)]
    pub(crate) fn reconstitute(snap: VehicleSnapshot) -> Self {
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

    pub fn replace_details(
        &mut self,
        number_plate: NumberPlate,
        description: Option<String>,
        water_capacity: WaterCapacity,
        status: VehicleStatus,
        vehicle_type: VehicleType,
        model: VehicleModel,
        driving_license: DrivingLicense,
        dimension: VehicleDimension,
        provenance: Provenance,
    ) {
        self.number_plate = number_plate;
        self.description = description;
        self.water_capacity = water_capacity;
        self.status = status;
        self.vehicle_type = vehicle_type;
        self.model = model;
        self.driving_license = driving_license;
        self.dimension = dimension;
        self.provenance = provenance;
    }

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
}
