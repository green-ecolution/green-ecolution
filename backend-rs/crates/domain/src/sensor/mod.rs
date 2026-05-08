//! Sensor aggregate — LoRaWAN (or similar) devices mounted on trees.
//!
//! The aggregate ([`Sensor`]) tracks connectivity status and physical location.
//! Time-series readings live in the [`data`] sub-module as the `SensorReading`
//! sub-aggregate. The view ([`SensorView`]) adds `created_at` / `updated_at`
//! audit fields and embeds the latest reading for HTTP responses.
//!
//! The `recorded_at` field on `SensorReading` is what the domain calls the
//! event timestamp; the DB column is named `created_at`, but the aggregate
//! exposes it as `recorded_at` to make the domain meaning explicit.

pub mod data;
pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use crate::shared::{
    coordinates::Coordinate,
    provenance::{Provenance, ProviderId},
};

pub use error::SensorError;
pub use repository::{SensorReader, SensorReadingReader, SensorReadingWriter, SensorWriter};
#[doc(hidden)]
pub use snapshot::SensorSnapshot;
pub use view::SensorView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "sensor_status", rename_all = "snake_case")
)]
pub enum SensorStatus {
    Online,
    Offline,
    Unknown,
}

crate::newtype_nonempty! {
    /// Sensor identifier (e.g. EUI from LoRaWAN), 1–64 characters after trimming.
    SensorId, "sensor.id", 1, 64
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sensor {
    pub id: SensorId,
    pub status: SensorStatus,
    pub coordinate: Coordinate,
    pub provenance: Provenance,
}

/// Input for creating a new [`Sensor`].
#[derive(Debug, Clone)]
pub struct SensorDraft {
    pub id: SensorId,
    pub status: SensorStatus,
    pub coordinate: Coordinate,
    pub provenance: Provenance,
}

impl Sensor {
    #[doc(hidden)]
    pub fn reconstitute(snap: SensorSnapshot) -> Self {
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
