//! Sensor aggregate — LoRaWAN (or similar) devices linked to trees.
//!
//! The aggregate ([`Sensor`]) tracks the provisioning lifecycle (`activated_at`,
//! `None` = prepared), sensor type, and model identity. Connectivity
//! (online/offline) is **derived** from reading recency via
//! [`derive_connectivity`], never stored. The physical position is no longer
//! part of the aggregate itself: location is derived from the linked tree.
//! Time-series readings live in the [`data`] sub-module as the `SensorReading`
//! sub-aggregate. The view ([`SensorView`]) adds `created_at` / `updated_at`
//! audit fields and embeds the latest reading for HTTP responses.
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

use chrono::{DateTime, Duration, Utc};

use crate::{
    Id,
    authorization::Visibility,
    organization::Organization,
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

/// Display-only connectivity state, derived via [`derive_connectivity`] —
/// never stored. `Prepared` mirrors `activated_at == None` on the aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    Prepared,
    Online,
    Offline,
}

/// Derives the connectivity shown to clients. Connectivity is never stored:
/// a sensor cannot enforce "I am online" — it is an observation about the
/// recency of its telemetry. Unactivated sensors stay `Prepared` regardless
/// of readings; the staleness boundary is inclusive.
pub fn derive_connectivity(
    activated_at: Option<DateTime<Utc>>,
    last_reading_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    offline_after: Duration,
) -> SensorStatus {
    if activated_at.is_none() {
        return SensorStatus::Prepared;
    }
    match last_reading_at {
        Some(at) if now - at <= offline_after => SensorStatus::Online,
        _ => SensorStatus::Offline,
    }
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
    activated_at: Option<DateTime<Utc>>,
    sensor_type: SensorType,
    model_id: Id<SensorModel>,
    lorawan: Option<LorawanCredentials>,
    organization_id: Id<Organization>,
}

/// Input for creating a new [`Sensor`].
#[derive(Debug, Clone)]
pub struct SensorDraft {
    pub id: SensorId,
    pub sensor_type: SensorType,
    pub model_id: Id<SensorModel>,
    pub provenance: Provenance,
    pub lorawan: LorawanCredentials,
    pub organization_id: Id<Organization>,
}

impl Sensor {
    #[doc(hidden)]
    pub fn reconstitute(snap: SensorSnapshot) -> Self {
        Self {
            id: SensorId::reconstitute(snap.id),
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
            activated_at: snap.activated_at,
            sensor_type: snap.sensor_type,
            model_id: Id::new(snap.model_id),
            lorawan: snap.lorawan,
            organization_id: Id::new(snap.organization_id),
        }
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn activated_at(&self) -> Option<DateTime<Utc>> {
        self.activated_at
    }

    pub fn is_activated(&self) -> bool {
        self.activated_at.is_some()
    }

    /// Prepared (never activated) -> activated at `at`, awaiting first reading.
    pub fn activate(
        &mut self,
        at: DateTime<Utc>,
    ) -> Result<Vec<crate::events::DomainEvent>, SensorError> {
        if self.is_activated() {
            return Err(SensorError::AlreadyActivated);
        }
        self.activated_at = Some(at);
        Ok(vec![crate::events::DomainEvent::SensorActivated {
            sensor_id: self.id.clone(),
        }])
    }

    /// Activated -> Prepared. Used when a sensor is taken out of service and
    /// its tree link removed. Inverse of [`Sensor::activate`].
    pub fn deactivate(&mut self) -> Result<Vec<crate::events::DomainEvent>, SensorError> {
        if !self.is_activated() {
            return Err(SensorError::NotActivated);
        }
        self.activated_at = None;
        Ok(vec![crate::events::DomainEvent::SensorDeactivated {
            sensor_id: self.id.clone(),
        }])
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

    /// Reassigns the sensor to `target`'s organization. No-op (and no event)
    /// if it already belongs there. Callers must ensure the sensor is not
    /// bound to a tree in a different organization (see
    /// `SensorService::transfer` and `TreeService::transfer`, which cascades
    /// the sensor of a transferred tree).
    pub fn transfer_to(&mut self, target: Id<Organization>) -> Vec<crate::events::DomainEvent> {
        if self.organization_id == target {
            return vec![];
        }
        let from = self.organization_id;
        self.organization_id = target;
        vec![
            crate::events::DomainEvent::SensorResponsibilityTransferred {
                sensor_id: self.id.clone(),
                from,
                to: target,
            },
        ]
    }
}

#[derive(Debug, Default, Clone)]
pub struct SensorSearchQuery {
    pub provider: Option<ProviderId>,
    /// Which organizations may see the result. Callers must set this per
    /// request; defaults to unrestricted for internal consumers.
    pub visible: Visibility,
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_now() -> chrono::DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap()
    }

    fn fixed_sensor() -> Sensor {
        Sensor {
            id: SensorId::new("eui-a81758fffe0c3b52").unwrap(),
            provenance: Provenance::default(),
            activated_at: Some(fixed_now()),
            sensor_type: SensorType::Lorawan,
            model_id: Id::new_v7(),
            lorawan: None,
            organization_id: Id::new_v7(),
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
    fn activate_sets_timestamp_and_emits_event() {
        use crate::events::DomainEvent;
        let mut s = fixed_sensor();
        s.activated_at = None;
        let at = fixed_now();
        let events = s.activate(at).unwrap();
        assert!(s.is_activated());
        assert_eq!(s.activated_at(), Some(at));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::SensorActivated { sensor_id } => {
                assert_eq!(sensor_id.as_str(), "eui-a81758fffe0c3b52")
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn activate_twice_returns_already_activated() {
        let mut s = fixed_sensor();
        let original = s.activated_at();
        assert!(matches!(
            s.activate(fixed_now()),
            Err(SensorError::AlreadyActivated)
        ));
        assert_eq!(s.activated_at(), original);
    }

    #[test]
    fn deactivate_clears_timestamp_and_emits_event() {
        use crate::events::DomainEvent;
        let mut s = fixed_sensor(); // fixture is already activated
        let events = s.deactivate().unwrap();
        assert!(!s.is_activated());
        assert_eq!(s.activated_at(), None);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::SensorDeactivated { sensor_id } => {
                assert_eq!(sensor_id.as_str(), "eui-a81758fffe0c3b52")
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn deactivate_when_not_activated_returns_not_activated() {
        let mut s = fixed_sensor();
        s.activated_at = None;
        assert!(matches!(s.deactivate(), Err(SensorError::NotActivated)));
        assert_eq!(s.activated_at(), None);
    }

    #[test]
    fn activate_after_deactivate_is_allowed() {
        let mut s = fixed_sensor();
        s.deactivate().unwrap();
        assert_ok!(s.activate(fixed_now()));
        assert!(s.is_activated());
    }

    #[test]
    fn transfer_to_same_org_is_noop() {
        use crate::organization::Organization;
        let mut s = fixed_sensor();
        let same: Id<Organization> = s.organization_id();
        let events = s.transfer_to(same);
        assert_eq!(s.organization_id(), same);
        assert!(events.is_empty());
    }

    #[test]
    fn transfer_to_new_org_emits_event_and_updates_field() {
        use crate::events::DomainEvent;
        use crate::organization::Organization;
        let mut s = fixed_sensor();
        let from = s.organization_id();
        let target: Id<Organization> = Id::new_v7();
        let events = s.transfer_to(target);
        assert_eq!(s.organization_id(), target);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::SensorResponsibilityTransferred {
                sensor_id,
                from: ev_from,
                to,
            } => {
                assert_eq!(sensor_id, &s.id);
                assert_eq!(*ev_from, from);
                assert_eq!(*to, target);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    mod derive_connectivity_tests {
        use super::super::*;
        use chrono::TimeZone;

        fn now() -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap()
        }

        fn threshold() -> Duration {
            Duration::hours(24)
        }

        #[test]
        fn unactivated_sensor_stays_prepared_even_with_fresh_reading() {
            let status =
                derive_connectivity(None, Some(now() - Duration::minutes(5)), now(), threshold());
            assert_eq!(status, SensorStatus::Prepared);
        }

        #[test]
        fn activated_sensor_with_fresh_reading_is_online() {
            let status = derive_connectivity(
                Some(now() - Duration::days(30)),
                Some(now() - Duration::hours(1)),
                now(),
                threshold(),
            );
            assert_eq!(status, SensorStatus::Online);
        }

        #[test]
        fn activated_sensor_with_stale_reading_is_offline() {
            let status = derive_connectivity(
                Some(now() - Duration::days(30)),
                Some(now() - Duration::hours(25)),
                now(),
                threshold(),
            );
            assert_eq!(status, SensorStatus::Offline);
        }

        #[test]
        fn activated_sensor_without_reading_is_offline() {
            let status =
                derive_connectivity(Some(now() - Duration::minutes(1)), None, now(), threshold());
            assert_eq!(status, SensorStatus::Offline);
        }

        #[test]
        fn reading_exactly_at_threshold_is_online() {
            let status = derive_connectivity(
                Some(now() - Duration::days(30)),
                Some(now() - Duration::hours(24)),
                now(),
                threshold(),
            );
            assert_eq!(status, SensorStatus::Online);
        }
    }
}
