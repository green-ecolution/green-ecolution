//! OperationPlan aggregate: a scheduled run of a module over a set of stops.
//!
//! ## State machine
//!
//! ```text
//! Planned ──start()──► Active ──finish(outcomes)──► Finished
//!    ▲                   │
//!    └──revert_start()───┤
//!    │                   │
//!    └──cancel(note)──┐  └──fail(note)──► NotCompleted
//!                     ▼
//!                  Canceled
//! ```
//!
//! Transitions not shown are rejected with [`OperationPlanError::InvalidStateTransition`].
//!
//! Plan content (`date`, `stops`, vehicles) may only be changed while
//! the plan is in `Planned`; `replace_details` enforces this via
//! [`OperationPlanError::CannotMutateAfterStart`].

pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    Id,
    authorization::Visibility,
    events::DomainEvent,
    module::ModuleKey,
    organization::Organization,
    planning::{StopOutcome, StopRef},
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        provenance::{Provenance, ProviderId},
    },
    start_point::StartPointName,
    vehicle::Vehicle,
};

pub use error::OperationPlanError;
pub use error::OperationPlanError as Error;
pub use repository::{OperationPlanReader, OperationPlanWriter};
#[doc(hidden)]
pub use snapshot::OperationPlanSnapshot;
pub use view::OperationPlanView;

/// Lifecycle status of an [`OperationPlan`].
///
/// See the module-level state machine diagram for valid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "operation_plan_status", rename_all = "snake_case")
)]
pub enum OperationPlanStatus {
    Planned,
    Active,
    Canceled,
    Finished,
    #[serde(rename = "not_completed", alias = "not competed")]
    #[cfg_attr(feature = "sqlx", sqlx(rename = "not_completed"))]
    NotCompleted,
}

/// Refill station a computed route visits, captured by name and location at
/// computation time so later depot edits don't rewrite history.
#[derive(Debug, Clone, PartialEq)]
pub struct StationVisit {
    pub name: StartPointName,
    pub coordinate: Coordinate,
}

/// Result of a route computation applied to a plan via
/// [`OperationPlan::set_metrics`]; [`RouteMetrics::cleared`] resets a plan to
/// the "no route" state.
#[derive(Debug, Clone, Default)]
pub struct RouteMetrics {
    pub distance: Option<Distance>,
    pub total_demand: Option<f64>,
    pub refill_count: u32,
    pub duration: Duration,
    pub route_geometry: Option<Vec<Coordinate>>,
    pub station_visits: Vec<StationVisit>,
}

impl RouteMetrics {
    pub fn cleared() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct OperationPlan {
    pub id: Id<OperationPlan>,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub distance: Option<Distance>,
    pub total_demand: Option<f64>,
    pub refill_count: u32,
    pub duration: Duration,

    module: ModuleKey,
    status: OperationPlanStatus,
    stops: Vec<StopRef>,
    transporter_id: Option<Id<Vehicle>>,
    trailer_id: Option<Id<Vehicle>>,
    transporter_capacity_used: bool,
    cancellation_note: Option<String>,
    provenance: Provenance,
    route_geometry: Option<Vec<Coordinate>>,
    station_visits: Vec<StationVisit>,
    user_ids: Vec<Uuid>,
    organization_id: Id<Organization>,
}

/// Input for creating a new [`OperationPlan`].
#[derive(Debug, Clone)]
pub struct OperationPlanDraft {
    pub module: ModuleKey,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub stops: Vec<StopRef>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub transporter_capacity_used: bool,
    pub provenance: Provenance,
    pub user_ids: Vec<Uuid>,
    pub organization_id: Id<Organization>,
}

impl OperationPlanDraft {
    pub fn validate(&self) -> Result<(), OperationPlanError> {
        ensure_capacity_available(self.trailer_id, self.transporter_capacity_used)
    }
}

/// Without a trailer the transporter's own capacity is the only one, so it
/// cannot be left out.
pub fn ensure_capacity_available(
    trailer_id: Option<Id<Vehicle>>,
    transporter_capacity_used: bool,
) -> Result<(), OperationPlanError> {
    if trailer_id.is_none() && !transporter_capacity_used {
        return Err(OperationPlanError::TransporterCapacityRequired);
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct OperationPlanSearchQuery {
    pub module: ModuleKey,
    pub provider: Option<ProviderId>,
    pub statuses: Vec<OperationPlanStatus>,
    /// Which organizations may see the result. Callers must set this per
    /// request.
    pub visible: Visibility,
}

/// Replacement input for [`OperationPlan`] field edits while still
/// in [`OperationPlanStatus::Planned`].
#[derive(Debug, Clone)]
pub struct OperationPlanUpdate {
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub stops: Vec<StopRef>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub transporter_capacity_used: bool,
    pub provenance: Provenance,
    pub user_ids: Vec<Uuid>,
}

impl OperationPlan {
    #[doc(hidden)]
    pub fn reconstitute(snap: OperationPlanSnapshot) -> Result<Self, OperationPlanError> {
        Ok(Self {
            id: Id::new(snap.id),
            date: snap.date,
            description: snap.description,
            start_point_name: snap.start_point_name,
            distance: snap.distance.and_then(|m| Distance::new(m).ok()),
            total_demand: snap.total_demand,
            refill_count: snap.refill_count.max(0) as u32,
            duration: snap.duration,
            module: ModuleKey::new(&snap.module)?,
            status: snap.status,
            stops: snap.stops.into_iter().map(StopRef::new).collect(),
            transporter_id: snap.transporter_id.map(Id::new),
            trailer_id: snap.trailer_id.map(Id::new),
            transporter_capacity_used: snap.transporter_capacity_used,
            cancellation_note: snap.cancellation_note,
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
            route_geometry: snap.route_geometry,
            station_visits: snap.station_visits,
            user_ids: snap.user_ids,
            organization_id: Id::new(snap.organization_id),
        })
    }

    pub fn module(&self) -> &ModuleKey {
        &self.module
    }

    pub fn status(&self) -> OperationPlanStatus {
        self.status
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn stops(&self) -> &[StopRef] {
        &self.stops
    }

    pub fn user_ids(&self) -> &[Uuid] {
        &self.user_ids
    }

    pub fn transporter_id(&self) -> Option<Id<Vehicle>> {
        self.transporter_id
    }

    pub fn trailer_id(&self) -> Option<Id<Vehicle>> {
        self.trailer_id
    }

    /// Whether the transporter's own capacity counts towards the route capacity.
    /// Only a plan with a trailer may leave it out.
    pub fn transporter_capacity_used(&self) -> bool {
        self.transporter_capacity_used
    }

    pub fn cancellation_note(&self) -> Option<&str> {
        self.cancellation_note.as_deref()
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn route_geometry(&self) -> Option<&[Coordinate]> {
        self.route_geometry.as_deref()
    }

    pub fn station_visits(&self) -> &[StationVisit] {
        &self.station_visits
    }

    fn ensure_planned(&self) -> Result<(), OperationPlanError> {
        if self.status != OperationPlanStatus::Planned {
            return Err(OperationPlanError::CannotMutateAfterStart);
        }
        Ok(())
    }

    fn event_scope(&self) -> (Id<OperationPlan>, ModuleKey, Vec<StopRef>) {
        (self.id, self.module.clone(), self.stops.clone())
    }

    /// Updates editable fields. Only allowed while status is
    /// [`OperationPlanStatus::Planned`]; once the plan starts, the only legal
    /// changes are status transitions, each of which emits its own event. No
    /// events are emitted here because edits to a planned plan have no
    /// cross-aggregate side effects.
    pub fn replace_details(
        &mut self,
        update: OperationPlanUpdate,
    ) -> Result<(), OperationPlanError> {
        self.ensure_planned()?;
        ensure_capacity_available(update.trailer_id, update.transporter_capacity_used)?;
        self.date = update.date;
        self.description = update.description;
        self.start_point_name = update.start_point_name;
        self.stops = update.stops;
        self.transporter_id = update.transporter_id;
        self.trailer_id = update.trailer_id;
        self.transporter_capacity_used = update.transporter_capacity_used;
        self.provenance = update.provenance;
        self.user_ids = update.user_ids;
        Ok(())
    }

    /// Transitions `Planned → Active`.
    pub fn start(&mut self) -> Result<Vec<DomainEvent>, OperationPlanError> {
        if self.status != OperationPlanStatus::Planned {
            return Err(OperationPlanError::InvalidStateTransition {
                from: self.status,
                to: OperationPlanStatus::Active,
            });
        }
        self.status = OperationPlanStatus::Active;
        let (plan_id, module, stops) = self.event_scope();
        Ok(vec![DomainEvent::OperationPlanStarted {
            plan_id,
            module,
            stops,
        }])
    }

    /// Transitions `Active → Planned`, undoing an accidental start.
    pub fn revert_start(&mut self) -> Result<Vec<DomainEvent>, OperationPlanError> {
        if self.status != OperationPlanStatus::Active {
            return Err(OperationPlanError::InvalidStateTransition {
                from: self.status,
                to: OperationPlanStatus::Planned,
            });
        }
        self.status = OperationPlanStatus::Planned;
        let (plan_id, module, stops) = self.event_scope();
        Ok(vec![DomainEvent::OperationPlanStartReverted {
            plan_id,
            module,
            stops,
        }])
    }

    /// Transitions `Planned | Active → Canceled`. `note` must be non-empty
    /// once trimmed.
    pub fn cancel(&mut self, note: String) -> Result<Vec<DomainEvent>, OperationPlanError> {
        if note.trim().is_empty() {
            return Err(OperationPlanError::CancellationNoteRequired);
        }
        if !matches!(
            self.status,
            OperationPlanStatus::Planned | OperationPlanStatus::Active
        ) {
            return Err(OperationPlanError::InvalidStateTransition {
                from: self.status,
                to: OperationPlanStatus::Canceled,
            });
        }
        self.status = OperationPlanStatus::Canceled;
        self.cancellation_note = Some(note);
        let (plan_id, module, stops) = self.event_scope();
        Ok(vec![DomainEvent::OperationPlanCanceled {
            plan_id,
            module,
            stops,
        }])
    }

    /// Transitions `Active → NotCompleted`. `note` must be non-empty once
    /// trimmed.
    pub fn fail(&mut self, note: String) -> Result<Vec<DomainEvent>, OperationPlanError> {
        if note.trim().is_empty() {
            return Err(OperationPlanError::CancellationNoteRequired);
        }
        if self.status != OperationPlanStatus::Active {
            return Err(OperationPlanError::InvalidStateTransition {
                from: self.status,
                to: OperationPlanStatus::NotCompleted,
            });
        }
        self.status = OperationPlanStatus::NotCompleted;
        self.cancellation_note = Some(note);
        let (plan_id, module, stops) = self.event_scope();
        Ok(vec![DomainEvent::OperationPlanFailed {
            plan_id,
            module,
            stops,
        }])
    }

    /// Transitions `Active → Finished`. Requires exactly one outcome per
    /// stop of the plan and none for other stops.
    pub fn finish(
        &mut self,
        outcomes: &[StopOutcome],
    ) -> Result<Vec<DomainEvent>, OperationPlanError> {
        if self.status != OperationPlanStatus::Active {
            return Err(OperationPlanError::InvalidStateTransition {
                from: self.status,
                to: OperationPlanStatus::Finished,
            });
        }
        let mut seen = std::collections::BTreeSet::new();
        for o in outcomes {
            if !self.stops.contains(&o.stop) {
                return Err(OperationPlanError::OutcomeUnknownStop(o.stop));
            }
            if !seen.insert(o.stop) {
                return Err(OperationPlanError::OutcomeDuplicate(o.stop));
            }
        }
        if let Some(missing) = self.stops.iter().find(|s| !seen.contains(*s)) {
            return Err(OperationPlanError::OutcomeMissing(*missing));
        }
        self.status = OperationPlanStatus::Finished;
        Ok(vec![DomainEvent::OperationPlanFinished {
            plan_id: self.id,
            module: self.module.clone(),
            stops: self.stops.clone(),
            finished_at: Utc::now(),
            outcomes: outcomes.to_vec(),
        }])
    }

    pub fn set_metrics(&mut self, metrics: RouteMetrics) {
        self.distance = metrics.distance;
        self.total_demand = metrics.total_demand;
        self.refill_count = metrics.refill_count;
        self.duration = metrics.duration;
        self.route_geometry = metrics.route_geometry;
        self.station_visits = metrics.station_visits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use claims::assert_err;

    fn stop(n: u128) -> StopRef {
        StopRef::new(Uuid::from_u128(n))
    }

    fn plan_with(stops: &[u128], status: OperationPlanStatus) -> OperationPlan {
        OperationPlan::reconstitute(OperationPlanSnapshot {
            id: Uuid::now_v7(),
            module: "irrigation".to_string(),
            date: Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap(),
            description: None,
            start_point_name: None,
            status,
            distance: None,
            total_demand: None,
            stops: stops.iter().map(|n| Uuid::from_u128(*n)).collect(),
            user_ids: vec![],
            transporter_id: Some(Uuid::now_v7()),
            trailer_id: None,
            transporter_capacity_used: true,
            cancellation_note: None,
            refill_count: 0,
            duration: Duration::default(),
            provider: None,
            additional_info: None,
            route_geometry: None,
            station_visits: Vec::new(),
            organization_id: Uuid::now_v7(),
        })
        .unwrap()
    }

    fn fixed_plan() -> (OperationPlan, [StopRef; 2]) {
        (
            plan_with(&[1, 2], OperationPlanStatus::Planned),
            [stop(1), stop(2)],
        )
    }

    fn outcome(n: u128, amount: f64) -> StopOutcome {
        StopOutcome {
            stop: stop(n),
            completed: true,
            amount: Some(amount),
        }
    }

    fn update_with_stops(stops: &[u128]) -> OperationPlanUpdate {
        OperationPlanUpdate {
            date: Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap(),
            description: None,
            start_point_name: None,
            stops: stops.iter().map(|n| stop(*n)).collect(),
            transporter_id: None,
            trailer_id: None,
            transporter_capacity_used: true,
            provenance: Provenance::default(),
            user_ids: vec![],
        }
    }

    #[test]
    fn start_from_planned_succeeds() {
        let (mut p, stops) = fixed_plan();
        let events = p.start().unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Active);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::OperationPlanStarted {
                plan_id,
                module,
                stops: event_stops,
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(module.as_str(), "irrigation");
                assert_eq!(event_stops, &stops.to_vec());
            }
            other => panic!("expected OperationPlanStarted, got {other:?}"),
        }
    }

    #[test]
    fn start_from_active_rejects() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        assert_err!(p.start());
    }

    #[test]
    fn cancel_requires_note() {
        let (mut p, _) = fixed_plan();
        assert_err!(p.cancel("".to_string()));
        assert_err!(p.cancel("   ".to_string()));
    }

    #[test]
    fn cancel_from_planned_succeeds() {
        let (mut p, stops) = fixed_plan();
        let events = p.cancel("not needed".to_string()).unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Canceled);
        assert_eq!(p.cancellation_note(), Some("not needed"));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::OperationPlanCanceled {
                plan_id,
                stops: event_stops,
                ..
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(event_stops, &stops.to_vec());
            }
            other => panic!("expected OperationPlanCanceled, got {other:?}"),
        }
    }

    #[test]
    fn cancel_from_active_succeeds() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        let events = p.cancel("aborted mid-run".to_string()).unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Canceled);
        assert_eq!(p.cancellation_note(), Some("aborted mid-run"));
        assert!(matches!(
            events[0],
            DomainEvent::OperationPlanCanceled { .. }
        ));
    }

    #[test]
    fn cancel_from_finished_rejects() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        p.finish(&[outcome(1, 100.0), outcome(2, 50.0)]).unwrap();
        assert_err!(p.cancel("too late".to_string()));
    }

    #[test]
    fn fail_only_from_active() {
        let (mut p, stops) = fixed_plan();
        assert_err!(p.fail("breakdown".to_string()));
        p.start().unwrap();
        let events = p.fail("breakdown".to_string()).unwrap();
        assert_eq!(p.status(), OperationPlanStatus::NotCompleted);
        assert_eq!(p.cancellation_note(), Some("breakdown"));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::OperationPlanFailed {
                plan_id,
                stops: event_stops,
                ..
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(event_stops, &stops.to_vec());
            }
            other => panic!("expected OperationPlanFailed, got {other:?}"),
        }
    }

    #[test]
    fn fail_requires_note() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        assert_err!(p.fail("".to_string()));
        assert_err!(p.fail("   ".to_string()));
    }

    #[test]
    fn finish_requires_active() {
        let (mut p, _) = fixed_plan();
        assert_err!(p.finish(&[]));
    }

    #[test]
    fn finish_requires_outcome_per_stop() {
        let (mut p, [_, s2]) = fixed_plan();
        p.start().unwrap();
        let err = p.finish(&[outcome(1, 100.0)]).unwrap_err();
        assert_eq!(err, OperationPlanError::OutcomeMissing(s2));
    }

    #[test]
    fn finish_succeeds_emits_event_when_all_stops_have_outcomes() {
        let (mut p, stops) = fixed_plan();
        p.start().unwrap();
        let events = p.finish(&[outcome(1, 100.0), outcome(2, 50.0)]).unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Finished);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::OperationPlanFinished {
                plan_id,
                stops: event_stops,
                outcomes,
                ..
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(event_stops, &stops.to_vec());
                assert_eq!(outcomes.len(), 2);
            }
            other => panic!("expected OperationPlanFinished, got {other:?}"),
        }
    }

    #[test]
    fn start_from_canceled_rejects() {
        let (mut p, _) = fixed_plan();
        p.cancel("done".to_string()).unwrap();
        assert_err!(p.start());
    }

    #[test]
    fn cancel_from_notcompleted_rejects() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        p.fail("breakdown".to_string()).unwrap();
        assert_err!(p.cancel("too late".to_string()));
    }

    #[test]
    fn replace_details_from_canceled_rejects() {
        let (mut p, _) = fixed_plan();
        p.cancel("nope".to_string()).unwrap();
        let result = p.replace_details(update_with_stops(&[]));
        assert_eq!(result, Err(OperationPlanError::CannotMutateAfterStart));
    }

    #[test]
    fn set_metrics_overwrites_run_results() {
        let (mut p, _) = fixed_plan();
        let dist = Distance::new(1234.0).unwrap();
        let geometry = vec![
            Coordinate::new(54.76, 9.43).unwrap(),
            Coordinate::new(54.80, 9.44).unwrap(),
        ];
        p.set_metrics(RouteMetrics {
            distance: Some(dist),
            total_demand: Some(99.5),
            refill_count: 3,
            duration: Duration::from_secs(60 * 45),
            route_geometry: Some(geometry.clone()),
            station_visits: Vec::new(),
        });
        assert_eq!(p.distance, Some(dist));
        assert_eq!(p.total_demand, Some(99.5));
        assert_eq!(p.refill_count, 3);
        assert_eq!(p.duration, Duration::from_secs(60 * 45));
        assert_eq!(p.route_geometry(), Some(geometry.as_slice()));
    }

    #[test]
    fn set_metrics_records_and_clears_station_visits() {
        let (mut p, _) = fixed_plan();
        let visit = StationVisit {
            name: StartPointName::new("Klärwerk Kielseng".to_string()).unwrap(),
            coordinate: Coordinate::new(54.8052, 9.4471).unwrap(),
        };
        p.set_metrics(RouteMetrics {
            refill_count: 1,
            duration: Duration::from_secs(60),
            station_visits: vec![visit.clone()],
            ..RouteMetrics::default()
        });
        assert_eq!(p.station_visits(), &[visit]);

        p.set_metrics(RouteMetrics::cleared());
        assert_eq!(p.station_visits(), &[]);
    }

    #[test]
    fn replace_details_sets_start_point_name() {
        let (mut p, _) = fixed_plan();
        p.replace_details(OperationPlanUpdate {
            start_point_name: Some("Betriebshof Schleswiger Straße".to_string()),
            transporter_id: p.transporter_id(),
            ..update_with_stops(&[1, 2])
        })
        .unwrap();
        assert_eq!(
            p.start_point_name.as_deref(),
            Some("Betriebshof Schleswiger Straße")
        );
    }

    #[test]
    fn replace_details_only_when_planned() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        let result = p.replace_details(OperationPlanUpdate {
            description: Some("new desc".to_string()),
            transporter_id: Some(Id::new_v7()),
            ..update_with_stops(&[9])
        });
        assert_eq!(result, Err(OperationPlanError::CannotMutateAfterStart));
    }

    #[test]
    fn revert_start_from_active_returns_to_planned() {
        let (mut p, stops) = fixed_plan();
        p.start().unwrap();
        let events = p.revert_start().unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Planned);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::OperationPlanStartReverted {
                plan_id,
                stops: event_stops,
                ..
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(event_stops, &stops.to_vec());
            }
            other => panic!("expected OperationPlanStartReverted, got {other:?}"),
        }
        p.start().unwrap();
        assert_eq!(p.status(), OperationPlanStatus::Active);
    }

    #[test]
    fn revert_start_only_from_active() {
        let (mut p, _) = fixed_plan();
        assert_err!(p.revert_start());
        p.start().unwrap();
        p.fail("breakdown".to_string()).unwrap();
        assert_err!(p.revert_start());
    }

    #[test]
    fn replace_details_updates_assigned_users() {
        let (mut p, _) = fixed_plan();
        let u1 = Uuid::now_v7();
        let u2 = Uuid::now_v7();
        p.replace_details(OperationPlanUpdate {
            user_ids: vec![u1, u2],
            ..update_with_stops(&[1, 2])
        })
        .unwrap();
        assert_eq!(p.user_ids(), &[u1, u2]);
    }

    fn update_with_vehicles(
        trailer_id: Option<Id<Vehicle>>,
        transporter_capacity_used: bool,
    ) -> OperationPlanUpdate {
        OperationPlanUpdate {
            trailer_id,
            transporter_capacity_used,
            ..update_with_stops(&[1, 2])
        }
    }

    #[test]
    fn replace_details_without_trailer_requires_transporter_capacity() {
        let (mut p, _) = fixed_plan();
        assert_eq!(
            p.replace_details(update_with_vehicles(None, false)),
            Err(OperationPlanError::TransporterCapacityRequired)
        );
        assert!(p.transporter_capacity_used());
    }

    #[test]
    fn replace_details_with_trailer_may_skip_transporter_capacity() {
        let (mut p, _) = fixed_plan();
        let trailer = Id::new_v7();
        p.replace_details(update_with_vehicles(Some(trailer), false))
            .unwrap();
        assert_eq!(p.trailer_id(), Some(trailer));
        assert!(!p.transporter_capacity_used());
    }

    fn draft_with_vehicles(
        trailer_id: Option<Id<Vehicle>>,
        transporter_capacity_used: bool,
    ) -> OperationPlanDraft {
        OperationPlanDraft {
            module: ModuleKey::new("irrigation").unwrap(),
            date: Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap(),
            description: None,
            start_point_name: None,
            stops: vec![],
            transporter_id: Some(Id::new_v7()),
            trailer_id,
            transporter_capacity_used,
            provenance: Provenance::default(),
            user_ids: vec![],
            organization_id: Id::new_v7(),
        }
    }

    #[test]
    fn draft_without_trailer_requires_transporter_capacity() {
        assert_eq!(
            draft_with_vehicles(None, false).validate(),
            Err(OperationPlanError::TransporterCapacityRequired)
        );
        assert_eq!(draft_with_vehicles(None, true).validate(), Ok(()));
    }

    #[test]
    fn draft_with_trailer_may_skip_transporter_capacity() {
        assert_eq!(
            draft_with_vehicles(Some(Id::new_v7()), false).validate(),
            Ok(())
        );
    }

    #[test]
    fn finish_needs_one_outcome_per_stop() {
        let mut plan = plan_with(&[1, 2], OperationPlanStatus::Active);
        let err = plan.finish(&[outcome(1, 10.0)]).unwrap_err();
        assert!(matches!(err, OperationPlanError::OutcomeMissing(s) if s == stop(2)));
        assert_eq!(plan.status(), OperationPlanStatus::Active);
    }

    #[test]
    fn finish_rejects_a_second_outcome_for_the_same_stop() {
        let mut plan = plan_with(&[1], OperationPlanStatus::Active);
        let err = plan
            .finish(&[outcome(1, 10.0), outcome(1, 5.0)])
            .unwrap_err();
        assert!(matches!(err, OperationPlanError::OutcomeDuplicate(_)));
    }

    #[test]
    fn finish_rejects_an_outcome_for_a_foreign_stop() {
        let mut plan = plan_with(&[1], OperationPlanStatus::Active);
        let err = plan
            .finish(&[outcome(1, 10.0), outcome(9, 5.0)])
            .unwrap_err();
        assert!(matches!(err, OperationPlanError::OutcomeUnknownStop(_)));
    }

    #[test]
    fn finish_carries_module_stops_and_outcomes() {
        let mut plan = plan_with(&[1], OperationPlanStatus::Active);
        let events = plan.finish(&[outcome(1, 10.0)]).unwrap();
        assert_eq!(plan.status(), OperationPlanStatus::Finished);
        match &events[0] {
            DomainEvent::OperationPlanFinished {
                module,
                stops,
                outcomes,
                ..
            } => {
                assert_eq!(module.as_str(), "irrigation");
                assert_eq!(stops.len(), 1);
                assert_eq!(outcomes[0].amount, Some(10.0));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn every_transition_outside_the_diagram_is_refused() {
        use OperationPlanStatus::*;
        for from in [Planned, Active, Canceled, Finished, NotCompleted] {
            let mut plan = plan_with(&[1], from);
            assert_eq!(plan.start().is_ok(), from == Planned);
            let mut plan = plan_with(&[1], from);
            assert_eq!(plan.revert_start().is_ok(), from == Active);
            let mut plan = plan_with(&[1], from);
            assert_eq!(
                plan.cancel("x".into()).is_ok(),
                matches!(from, Planned | Active)
            );
            let mut plan = plan_with(&[1], from);
            assert_eq!(plan.fail("x".into()).is_ok(), from == Active);
            let mut plan = plan_with(&[1], from);
            assert_eq!(plan.finish(&[outcome(1, 1.0)]).is_ok(), from == Active);
        }
    }

    #[test]
    fn a_refused_transition_leaves_the_plan_untouched() {
        let mut plan = plan_with(&[1], OperationPlanStatus::Finished);
        assert!(plan.cancel("x".into()).is_err());
        assert_eq!(plan.status(), OperationPlanStatus::Finished);
        assert_eq!(plan.cancellation_note(), None);
    }

    #[test]
    fn details_are_frozen_after_start() {
        let mut plan = plan_with(&[1], OperationPlanStatus::Active);
        let err = plan.replace_details(update_with_stops(&[2])).unwrap_err();
        assert!(matches!(err, OperationPlanError::CannotMutateAfterStart));
    }

    #[test]
    fn without_trailer_the_transporter_capacity_is_required() {
        assert!(matches!(
            ensure_capacity_available(None, false),
            Err(OperationPlanError::TransporterCapacityRequired)
        ));
        assert!(ensure_capacity_available(None, true).is_ok());
        assert!(ensure_capacity_available(Some(Id::new_v7()), false).is_ok());
    }

    #[test]
    fn reconstitute_rejects_an_invalid_module_key() {
        let err = OperationPlan::reconstitute(OperationPlanSnapshot {
            id: Uuid::now_v7(),
            module: String::new(),
            date: Utc::now(),
            description: None,
            start_point_name: None,
            status: OperationPlanStatus::Planned,
            distance: Some(-1.0),
            total_demand: None,
            stops: vec![],
            user_ids: vec![],
            transporter_id: None,
            trailer_id: None,
            transporter_capacity_used: true,
            cancellation_note: None,
            refill_count: -3,
            duration: Duration::default(),
            provider: None,
            additional_info: None,
            route_geometry: None,
            station_visits: vec![],
            organization_id: Uuid::now_v7(),
        })
        .unwrap_err();
        assert!(matches!(err, OperationPlanError::Validation(_)));
    }
}
