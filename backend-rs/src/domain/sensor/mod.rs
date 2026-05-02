pub mod data;
pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use crate::domain::shared::{
    coordinates::Coordinate,
    error::ValidationError,
    provenance::{Provenance, ProviderId},
    string_value::NonEmptyString,
};

pub use error::SensorError;
pub use repository::{SensorReader, SensorReadingReader, SensorReadingWriter, SensorWriter};
#[allow(unused_imports)]
pub(crate) use snapshot::SensorSnapshot;
pub use view::SensorView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "sensor_status", rename_all = "snake_case")]
pub enum SensorStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorId(NonEmptyString);

impl SensorId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(value, "sensor.id", 1, 64)?))
    }

    pub(crate) fn reconstitute(value: String) -> Self {
        Self(NonEmptyString::reconstitute(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for SensorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sensor {
    pub id: SensorId,
    pub status: SensorStatus,
    pub coordinate: Coordinate,
    pub provenance: Provenance,
}

#[derive(Debug, Clone)]
pub struct SensorDraft {
    pub id: SensorId,
    pub status: SensorStatus,
    pub coordinate: Coordinate,
    pub provenance: Provenance,
}

impl Sensor {
    #[allow(dead_code)]
    pub(crate) fn reconstitute(snap: SensorSnapshot) -> Self {
        Self {
            id: SensorId::reconstitute(snap.id),
            status: snap.status,
            coordinate: Coordinate::new(snap.latitude, snap.longitude).expect(
                "DB coordinate values must be valid; row was persisted only after validation",
            ),
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
        }
    }

    pub fn change_status(&mut self, new: SensorStatus) {
        if self.status == new {
            return;
        }
        self.status = new;
    }

    pub fn move_to(&mut self, new: Coordinate) {
        if self.coordinate == new {
            return;
        }
        self.coordinate = new;
    }
}

#[derive(Debug, Default, Clone)]
pub struct SensorSearchQuery {
    pub provider: Option<ProviderId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_sensor() -> Sensor {
        Sensor {
            id: SensorId::new("eui-a81758fffe0c3b52").unwrap(),
            status: SensorStatus::Online,
            coordinate: Coordinate::new(54.7937, 9.4469).unwrap(),
            provenance: Provenance::default(),
        }
    }

    #[test]
    fn sensor_id_rejects_empty() {
        assert_err!(SensorId::new(""));
    }

    #[test]
    fn sensor_id_accepts_valid() {
        assert_ok!(SensorId::new("eui-a81758fffe0c3b52"));
    }

    #[test]
    fn sensor_id_rejects_too_long() {
        let long: String = "x".repeat(65);
        assert_err!(SensorId::new(long));
    }

    #[test]
    fn change_status_to_same_is_noop() {
        let mut s = fixed_sensor();
        s.change_status(SensorStatus::Online);
        assert_eq!(s.status, SensorStatus::Online);
    }

    #[test]
    fn change_status_to_different_changes_status() {
        let mut s = fixed_sensor();
        s.change_status(SensorStatus::Offline);
        assert_eq!(s.status, SensorStatus::Offline);
    }

    #[test]
    fn move_to_same_coord_is_noop() {
        let mut s = fixed_sensor();
        let original = s.coordinate;
        s.move_to(Coordinate::new(54.7937, 9.4469).unwrap());
        assert_eq!(s.coordinate, original);
    }

    #[test]
    fn move_to_new_coord_changes_position() {
        let mut s = fixed_sensor();
        let new = Coordinate::new(53.0, 9.0).unwrap();
        s.move_to(new);
        assert_eq!(s.coordinate, new);
    }
}
