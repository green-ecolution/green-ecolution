//! Sensor aggregate — LoRaWAN (or similar) devices attached to a module's
//! target, such as a tree.
//!
//! The aggregate ([`Sensor`]) tracks the provisioning lifecycle (`activated_at`,
//! `None` = prepared), sensor type, model identity and the target it measures
//! for. Connectivity (online/offline) is **derived** from reading recency via
//! [`derive_connectivity`], never stored. Location and label come from the
//! target's module.
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
pub mod plausibility;
pub mod repository;
pub mod snapshot;
pub mod sort;
pub mod view;

use chrono::{DateTime, Duration, Utc};

use crate::{
    Id,
    authorization::Visibility,
    events::DomainEvent,
    module::ModuleKey,
    organization::Organization,
    sensor_model::SensorModel,
    shared::provenance::{Provenance, ProviderId},
    target::TargetRef,
};

pub use error::SensorError;
pub use plausibility::{
    DataHealth, PlausibilityIssue, PlausibilityReason, ReadingContext, ReadingQualityIssue,
    derive_data_health,
};
pub use repository::{
    LastPlausibleValue, NormalizedValue, SensorReader, SensorReadingReader, SensorReadingWriter,
    SensorWriter,
};
#[doc(hidden)]
pub use snapshot::SensorSnapshot;
pub use sort::{SensorSort, SensorSortField};
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

crate::newtype_nonempty! {
    /// Reason someone recorded when reviewing a sensor's flagged readings.
    AcknowledgementNote, "sensor.acknowledgement.note", 1, 500
}

/// Records that someone reviewed this sensor's flagged readings. `at` is a
/// watermark: values flagged after it are unreviewed and raise the warning
/// again without anyone having to re-arm it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataQualityAcknowledgement {
    pub at: DateTime<Utc>,
    pub by: uuid::Uuid,
    pub note: Option<AcknowledgementNote>,
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
    quality_acknowledged: Option<DataQualityAcknowledgement>,
    target: Option<TargetRef>,
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
            quality_acknowledged: snap.quality_acknowledged,
            target: snap.target,
        }
    }

    pub fn target(&self) -> Option<&TargetRef> {
        self.target.as_ref()
    }

    pub fn quality_acknowledged(&self) -> Option<&DataQualityAcknowledgement> {
        self.quality_acknowledged.as_ref()
    }

    /// Marks every reading flagged up to `ack.at` as reviewed. The watermark
    /// only moves forward: a request carrying an older timestamp is ignored so
    /// a late or replayed call cannot reopen warnings someone already closed.
    pub fn acknowledge_data_quality(
        &mut self,
        ack: DataQualityAcknowledgement,
    ) -> Vec<crate::events::DomainEvent> {
        if let Some(current) = &self.quality_acknowledged
            && ack.at <= current.at
        {
            return vec![];
        }
        let event = crate::events::DomainEvent::SensorDataQualityAcknowledged {
            sensor_id: self.id.clone(),
            at: ack.at,
            by: ack.by,
        };
        self.quality_acknowledged = Some(ack);
        vec![event]
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

    /// Prepared -> activated at `at` and attached to `target`, awaiting the
    /// first reading. A sensor already attached elsewhere is refused.
    pub fn activate(
        &mut self,
        target: TargetRef,
        at: DateTime<Utc>,
    ) -> Result<Vec<DomainEvent>, SensorError> {
        if self.activated_at.is_some() {
            return Err(SensorError::AlreadyActivated);
        }
        if self.target.as_ref().is_some_and(|t| *t != target) {
            return Err(SensorError::AlreadyAttached);
        }
        self.activated_at = Some(at);
        let mut events = vec![DomainEvent::SensorActivated {
            sensor_id: self.id.clone(),
        }];
        if self.target.is_none() {
            self.target = Some(target.clone());
            events.push(DomainEvent::SensorAttached {
                sensor_id: self.id.clone(),
                target,
            });
        }
        Ok(events)
    }

    /// Attaches without activating, so a prepared sensor can be assigned
    /// before it goes into service.
    pub fn attach(&mut self, target: TargetRef) -> Result<Vec<DomainEvent>, SensorError> {
        match &self.target {
            Some(current) if *current == target => Ok(vec![]),
            Some(_) => Err(SensorError::AlreadyAttached),
            None => {
                self.target = Some(target.clone());
                Ok(vec![DomainEvent::SensorAttached {
                    sensor_id: self.id.clone(),
                    target,
                }])
            }
        }
    }

    /// Moves an active sensor to `target`, releasing its current one.
    pub fn retarget(&mut self, target: TargetRef) -> Result<Vec<DomainEvent>, SensorError> {
        if self.activated_at.is_none() {
            return Err(SensorError::NotActivated);
        }
        if self.target.as_ref() == Some(&target) {
            return Ok(vec![]);
        }
        let mut events = self.detach();
        self.target = Some(target.clone());
        events.push(DomainEvent::SensorAttached {
            sensor_id: self.id.clone(),
            target,
        });
        Ok(events)
    }

    pub fn detach(&mut self) -> Vec<DomainEvent> {
        match self.target.take() {
            Some(target) => vec![DomainEvent::SensorDetached {
                sensor_id: self.id.clone(),
                target,
            }],
            None => vec![],
        }
    }

    /// Activated -> Prepared, releasing the target. Inverse of
    /// [`Sensor::activate`].
    pub fn deactivate(&mut self) -> Result<Vec<DomainEvent>, SensorError> {
        if self.activated_at.is_none() {
            return Err(SensorError::NotActivated);
        }
        self.activated_at = None;
        let mut events = vec![DomainEvent::SensorDeactivated {
            sensor_id: self.id.clone(),
        }];
        events.extend(self.detach());
        Ok(events)
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
    /// attached to a target in a different organization (see
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
    // Scope
    pub provider: Option<ProviderId>,
    /// Which organizations may see the result. Callers must set this per
    /// request; defaults to unrestricted for internal consumers.
    pub visible: Visibility,

    // Filters
    /// Connectivity, which is derived rather than stored — the repository has
    /// to reproduce [`derive_connectivity`] in SQL to filter on it.
    pub statuses: Vec<SensorStatus>,
    pub model_ids: Vec<Id<SensorModel>>,
    /// Data health, derived the same way from [`derive_data_health`].
    pub data_health: Vec<DataHealth>,
    /// Whether the sensor is attached to a target.
    pub attached: Option<bool>,
    /// Only sensors attached to a target of this module.
    pub target_module: Option<ModuleKey>,
    pub ids: Vec<SensorId>,

    // Search and order
    /// Case-insensitive text filter on sensor id or model name.
    pub q: Option<String>,
    pub sort: SensorSort,
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
            quality_acknowledged: None,
            target: None,
        }
    }

    fn prepared_sensor() -> Sensor {
        let mut s = fixed_sensor();
        s.activated_at = None;
        s
    }

    fn tree(id: u128) -> TargetRef {
        TargetRef::new("irrigation", "tree", uuid::Uuid::from_u128(id)).unwrap()
    }

    #[test]
    fn activate_attaches_and_emits_both_events() {
        let mut s = prepared_sensor();
        let events = s.activate(tree(1), Utc::now()).unwrap();
        assert_eq!(s.target(), Some(&tree(1)));
        assert!(matches!(events[0], DomainEvent::SensorActivated { .. }));
        assert!(
            matches!(&events[1], DomainEvent::SensorAttached { target, .. } if *target == tree(1))
        );
    }

    #[test]
    fn activate_twice_is_refused() {
        let mut s = prepared_sensor();
        s.activate(tree(1), Utc::now()).unwrap();
        assert!(matches!(
            s.activate(tree(1), Utc::now()),
            Err(SensorError::AlreadyActivated)
        ));
    }

    #[test]
    fn activate_onto_another_target_than_the_attached_one_is_refused() {
        let mut s = prepared_sensor();
        s.attach(tree(1)).unwrap();
        assert!(matches!(
            s.activate(tree(2), Utc::now()),
            Err(SensorError::AlreadyAttached)
        ));
    }

    #[test]
    fn activate_onto_the_attached_target_emits_no_second_attach() {
        let mut s = prepared_sensor();
        s.attach(tree(1)).unwrap();
        let events = s.activate(tree(1), Utc::now()).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::SensorActivated { .. }));
    }

    #[test]
    fn attach_without_activation_keeps_the_sensor_prepared() {
        let mut s = prepared_sensor();
        let events = s.attach(tree(1)).unwrap();
        assert!(s.activated_at().is_none());
        assert_eq!(events.len(), 1);
        assert!(s.attach(tree(1)).unwrap().is_empty());
        assert!(matches!(
            s.attach(tree(2)),
            Err(SensorError::AlreadyAttached)
        ));
    }

    #[test]
    fn retarget_moves_an_active_sensor() {
        let mut s = prepared_sensor();
        s.activate(tree(1), Utc::now()).unwrap();
        let events = s.retarget(tree(2)).unwrap();
        assert!(
            matches!(&events[0], DomainEvent::SensorDetached { target, .. } if *target == tree(1))
        );
        assert!(
            matches!(&events[1], DomainEvent::SensorAttached { target, .. } if *target == tree(2))
        );
        assert!(s.retarget(tree(2)).unwrap().is_empty());
    }

    #[test]
    fn retarget_needs_an_active_sensor() {
        let mut s = prepared_sensor();
        assert!(matches!(
            s.retarget(tree(1)),
            Err(SensorError::NotActivated)
        ));
    }

    #[test]
    fn detach_is_a_no_op_without_target() {
        let mut s = prepared_sensor();
        assert!(s.detach().is_empty());
        s.attach(tree(1)).unwrap();
        assert_eq!(s.detach().len(), 1);
        assert_eq!(s.target(), None);
    }

    #[test]
    fn deactivate_releases_the_target() {
        let mut s = prepared_sensor();
        s.activate(tree(1), Utc::now()).unwrap();
        let events = s.deactivate().unwrap();
        assert!(s.activated_at().is_none());
        assert_eq!(s.target(), None);
        assert!(matches!(events[0], DomainEvent::SensorDeactivated { .. }));
        assert!(
            matches!(&events[1], DomainEvent::SensorDetached { target, .. } if *target == tree(1))
        );
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
        let events = s.activate(tree(1), at).unwrap();
        assert!(s.is_activated());
        assert_eq!(s.activated_at(), Some(at));
        assert_eq!(events.len(), 2);
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
            s.activate(tree(1), fixed_now()),
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
        assert_ok!(s.activate(tree(1), fixed_now()));
        assert!(s.is_activated());
    }

    fn ack(at: DateTime<Utc>, note: Option<&str>) -> DataQualityAcknowledgement {
        DataQualityAcknowledgement {
            at,
            by: uuid::Uuid::now_v7(),
            note: note.map(|n| AcknowledgementNote::new(n).unwrap()),
        }
    }

    #[test]
    fn acknowledging_sets_the_watermark_and_emits_an_event() {
        use crate::events::DomainEvent;
        let mut s = fixed_sensor();
        assert!(s.quality_acknowledged().is_none());

        let at = fixed_now();
        let events = s.acknowledge_data_quality(ack(at, Some("Sonde nicht angeschlossen")));

        assert_eq!(s.quality_acknowledged().map(|a| a.at), Some(at));
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            DomainEvent::SensorDataQualityAcknowledged { sensor_id, .. } if sensor_id == &s.id
        ));
    }

    #[test]
    fn acknowledging_again_moves_the_watermark_forward() {
        let mut s = fixed_sensor();
        s.acknowledge_data_quality(ack(fixed_now(), None));
        let later = fixed_now() + Duration::hours(1);

        let events = s.acknowledge_data_quality(ack(later, Some("erneut geprüft")));

        assert_eq!(s.quality_acknowledged().map(|a| a.at), Some(later));
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn acknowledging_with_an_older_timestamp_is_a_noop() {
        let mut s = fixed_sensor();
        let current = fixed_now();
        s.acknowledge_data_quality(ack(current, Some("aktuell")));

        let events =
            s.acknowledge_data_quality(ack(current - Duration::hours(2), Some("veraltet")));

        assert_eq!(
            s.quality_acknowledged().map(|a| a.at),
            Some(current),
            "a late request must not reopen already reviewed warnings"
        );
        assert_eq!(
            s.quality_acknowledged()
                .and_then(|a| a.note.clone())
                .map(|n| n.as_str().to_owned()),
            Some("aktuell".to_owned())
        );
        assert!(events.is_empty());
    }

    #[test]
    fn acknowledging_with_the_same_timestamp_is_a_noop() {
        let mut s = fixed_sensor();
        let at = fixed_now();
        s.acknowledge_data_quality(ack(at, None));
        assert!(s.acknowledge_data_quality(ack(at, None)).is_empty());
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
