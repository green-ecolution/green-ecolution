//! Sensor aggregate — LoRaWAN (or similar) devices linked to trees.
//!
//! The aggregate ([`Sensor`]) tracks connectivity status, sensor type, and
//! model identity. The physical position is no longer part of the aggregate
//! itself: location is derived from the linked tree. Time-series readings live
//! in the [`data`] sub-module as the `SensorReading` sub-aggregate. The view
//! ([`SensorView`]) adds `created_at` / `updated_at` audit fields and embeds
//! the latest reading for HTTP responses.
//!
//! The `recorded_at` field on `SensorReading` is what the domain calls the
//! event timestamp; the DB column is named `created_at`, but the aggregate
//! exposes it as `recorded_at` to make the domain meaning explicit.

pub mod data;
pub mod error;
pub mod payload;
pub mod repository;
pub mod snapshot;
pub mod view;

use crate::{
    Id,
    sensor_model::SensorModel,
    shared::provenance::{Provenance, ProviderId},
};

pub use error::SensorError;
pub use repository::{
    NormalizedValue, SensorReader, SensorReadingReader, SensorReadingWriter, SensorWriter,
};
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
    Prepared,
    Online,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "sensor_type", rename_all = "snake_case")
)]
pub enum SensorType {
    Lorawan,
}

#[derive(Debug, Clone)]
pub struct LorawanCredentials {
    pub serial_number: crate::shared::string_value::NonEmptyString,
    pub dev_eui: crate::shared::string_value::NonEmptyString,
    pub app_eui: crate::shared::string_value::NonEmptyString,
    pub app_key: secrecy::SecretString,
    pub at_pin: Option<String>,
    pub ota_pin: Option<String>,
    pub config: Option<serde_json::Value>,
}

crate::newtype_nonempty! {
    /// Sensor identifier (e.g. EUI from LoRaWAN), 1–64 characters after trimming.
    SensorId, "sensor.id", 1, 64
}

#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub provenance: Provenance,
    status: SensorStatus,
    sensor_type: SensorType,
    model_id: Id<SensorModel>,
    lorawan: Option<LorawanCredentials>,
}

/// Input for creating a new [`Sensor`].
#[derive(Debug, Clone)]
pub struct SensorDraft {
    pub id: SensorId,
    pub sensor_type: SensorType,
    pub model_id: Id<SensorModel>,
    pub provenance: Provenance,
    pub lorawan: LorawanCredentials,
}

impl Sensor {
    #[doc(hidden)]
    pub fn reconstitute(snap: SensorSnapshot) -> Self {
        Self {
            id: SensorId::reconstitute(snap.id),
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
            status: snap.status,
            sensor_type: snap.sensor_type,
            model_id: Id::new(snap.model_id),
            lorawan: snap.lorawan,
        }
    }

    pub fn status(&self) -> SensorStatus {
        self.status
    }

    pub fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    pub fn model_id(&self) -> Id<SensorModel> {
        self.model_id
    }

    pub fn lorawan(&self) -> Option<&LorawanCredentials> {
        self.lorawan.as_ref()
    }

    /// Prepared -> Offline (activated, awaiting first reading).
    pub fn activate(&mut self) -> Result<Vec<crate::events::DomainEvent>, SensorError> {
        match self.status {
            SensorStatus::Prepared => {
                self.status = SensorStatus::Offline;
                Ok(vec![crate::events::DomainEvent::SensorActivated {
                    sensor_id: self.id.clone(),
                }])
            }
            _ => Err(SensorError::AlreadyActivated),
        }
    }

    pub fn change_status(&mut self, new: SensorStatus) {
        if self.status == new {
            return;
        }
        self.status = new;
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
            provenance: Provenance::default(),
            status: SensorStatus::Online,
            sensor_type: SensorType::Lorawan,
            model_id: Id::new(1),
            lorawan: None,
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
        assert_eq!(s.status(), SensorStatus::Online);
    }

    #[test]
    fn change_status_to_different_changes_status() {
        let mut s = fixed_sensor();
        s.change_status(SensorStatus::Offline);
        assert_eq!(s.status(), SensorStatus::Offline);
    }

    #[test]
    fn activate_from_prepared_transitions_to_offline_and_emits_event() {
        use crate::events::DomainEvent;
        let mut s = fixed_sensor();
        s.status = SensorStatus::Prepared;
        let events = s.activate().unwrap();
        assert_eq!(s.status(), SensorStatus::Offline);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::SensorActivated { sensor_id } => {
                assert_eq!(sensor_id.as_str(), "eui-a81758fffe0c3b52")
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn activate_from_non_prepared_returns_already_activated() {
        let mut s = fixed_sensor();
        s.status = SensorStatus::Online;
        assert!(matches!(s.activate(), Err(SensorError::AlreadyActivated)));
        assert_eq!(s.status(), SensorStatus::Online);
    }
}
