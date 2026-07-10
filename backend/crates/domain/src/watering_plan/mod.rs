//! WateringPlan aggregate — a scheduled watering run for a set of tree clusters.
//!
//! ## State machine
//!
//! ```text
//! Planned ──start()──► Active ──finish(evals)──► Finished
//!    ▲                   │
//!    └──revert_start()───┤
//!    │                   │
//!    └──cancel(note)──┐  └──fail(note)──► NotCompleted
//!                     ▼
//!                  Canceled
//! ```
//!
//! Transitions not shown are rejected with [`WateringPlanError::InvalidStateTransition`].
//! `Unknown` is a legacy/sentinel value from the DB; it does not participate
//! in normal transitions.
//!
//! Plan content (`date`, `cluster_ids`, vehicles) may only be changed while
//! the plan is in `Planned`; `replace_details` enforces this via
//! [`WateringPlanError::CannotMutateAfterStart`].

pub mod error;
pub mod evaluation;
pub mod repository;
pub mod snapshot;
pub mod view;

use std::time::Duration;

use chrono::{DateTime, Utc};
use url::Url;

use crate::{
    Id,
    cluster::TreeCluster,
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        provenance::{Provenance, ProviderId},
    },
    vehicle::Vehicle,
};

pub use error::WateringPlanError;
pub use error::WateringPlanError as Error;
pub use evaluation::WateringPlanEvaluation;
pub use repository::{WateringPlanReader, WateringPlanWriter};
#[doc(hidden)]
pub use snapshot::WateringPlanSnapshot;
pub use view::WateringPlanView;

/// Lifecycle status of a [`WateringPlan`].
///
/// See the module-level state machine diagram for valid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "watering_plan_status", rename_all = "snake_case")
)]
pub enum WateringPlanStatus {
    Planned,
    Active,
    Canceled,
    Finished,
    #[serde(rename = "not competed")]
    #[cfg_attr(feature = "sqlx", sqlx(rename = "not competed"))]
    NotCompleted,
    Unknown,
}

/// Water refill station a computed route visits, captured by name and
/// location at computation time so later depot edits don't rewrite history.
#[derive(Debug, Clone, PartialEq)]
pub struct RefillPoint {
    pub name: crate::start_point::StartPointName,
    pub coordinate: Coordinate,
}

/// Result of a route computation applied to a plan via
/// [`WateringPlan::set_metrics`]; [`RouteMetrics::cleared`] resets a plan to
/// the "no route" state.
#[derive(Debug, Clone, Default)]
pub struct RouteMetrics {
    pub distance: Option<Distance>,
    pub total_water_required: Option<f64>,
    pub refill_count: u32,
    pub duration: Duration,
    pub gpx_url: Option<Url>,
    pub route_geometry: Option<Vec<Coordinate>>,
    pub refill_points: Vec<RefillPoint>,
}

impl RouteMetrics {
    pub fn cleared() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WateringPlan {
    pub id: Id<WateringPlan>,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub distance: Option<Distance>,
    pub total_water_required: Option<f64>,
    pub gpx_url: Option<Url>,
    pub refill_count: u32,
    pub duration: Duration,

    status: WateringPlanStatus,
    cluster_ids: Vec<Id<TreeCluster>>,
    transporter_id: Option<Id<Vehicle>>,
    trailer_id: Option<Id<Vehicle>>,
    cancellation_note: Option<String>,
    provenance: Provenance,
    route_geometry: Option<Vec<Coordinate>>,
    refill_points: Vec<RefillPoint>,
    user_ids: Vec<uuid::Uuid>,
}

/// Input for creating a new [`WateringPlan`].
#[derive(Debug, Clone)]
pub struct WateringPlanDraft {
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub provenance: Provenance,
    pub user_ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Default, Clone)]
pub struct WateringPlanSearchQuery {
    pub provider: Option<ProviderId>,
    pub statuses: Vec<WateringPlanStatus>,
}

/// Replacement input for [`WateringPlan`] field edits while still
/// in [`WateringPlanStatus::Planned`].
#[derive(Debug, Clone)]
pub struct WateringPlanUpdate {
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub provenance: Provenance,
    pub user_ids: Vec<uuid::Uuid>,
}

impl WateringPlan {
    #[doc(hidden)]
    pub fn reconstitute(snap: WateringPlanSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            date: snap.date,
            description: snap.description,
            start_point_name: snap.start_point_name,
            distance: snap.distance.and_then(|m| Distance::new(m).ok()),
            total_water_required: snap.total_water_required,
            gpx_url: snap.gpx_url,
            refill_count: snap.refill_count.max(0) as u32,
            duration: snap.duration,
            status: snap.status,
            cluster_ids: snap.cluster_ids.into_iter().map(Id::new).collect(),
            transporter_id: snap.transporter_id.map(Id::new),
            trailer_id: snap.trailer_id.map(Id::new),
            cancellation_note: snap.cancellation_note,
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
            route_geometry: snap.route_geometry,
            refill_points: snap.refill_points,
            user_ids: snap.user_ids,
        }
    }

    pub fn status(&self) -> WateringPlanStatus {
        self.status
    }

    pub fn cluster_ids(&self) -> &[Id<TreeCluster>] {
        &self.cluster_ids
    }

    pub fn user_ids(&self) -> &[uuid::Uuid] {
        &self.user_ids
    }

    pub fn transporter_id(&self) -> Option<Id<Vehicle>> {
        self.transporter_id
    }

    pub fn trailer_id(&self) -> Option<Id<Vehicle>> {
        self.trailer_id
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

    pub fn refill_points(&self) -> &[RefillPoint] {
        &self.refill_points
    }

    fn ensure_planned(&self) -> Result<(), WateringPlanError> {
        if self.status != WateringPlanStatus::Planned {
            return Err(WateringPlanError::CannotMutateAfterStart);
        }
        Ok(())
    }

    /// Updates editable fields (date, description, cluster set, vehicles,
    /// provenance). Only allowed while status is
    /// [`WateringPlanStatus::Planned`] — once the plan starts, the only legal
    /// changes are status transitions (`start`/`cancel`/`fail`/`finish`),
    /// each of which emits its own event. No events are emitted here because
    /// edits to a planned plan have no cross-aggregate side effects.
    pub fn replace_details(&mut self, update: WateringPlanUpdate) -> Result<(), WateringPlanError> {
        self.ensure_planned()?;
        self.date = update.date;
        self.description = update.description;
        self.start_point_name = update.start_point_name;
        self.cluster_ids = update.cluster_ids;
        self.transporter_id = update.transporter_id;
        self.trailer_id = update.trailer_id;
        self.provenance = update.provenance;
        self.user_ids = update.user_ids;
        Ok(())
    }

    /// Transitions `Planned → Active`. Fails on any other starting status.
    pub fn start(&mut self) -> Result<Vec<DomainEvent>, WateringPlanError> {
        if self.status != WateringPlanStatus::Planned {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::Active,
            });
        }
        self.status = WateringPlanStatus::Active;
        Ok(vec![DomainEvent::WateringPlanStarted {
            plan_id: self.id,
            cluster_ids: self.cluster_ids.clone(),
        }])
    }

    /// Transitions `Active → Planned`, undoing an accidental start.
    pub fn revert_start(&mut self) -> Result<Vec<DomainEvent>, WateringPlanError> {
        if self.status != WateringPlanStatus::Active {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::Planned,
            });
        }
        self.status = WateringPlanStatus::Planned;
        Ok(vec![DomainEvent::WateringPlanStartReverted {
            plan_id: self.id,
            cluster_ids: self.cluster_ids.clone(),
        }])
    }

    /// Transitions `Planned | Active → Canceled`.
    ///
    /// `note` must be non-empty (trimmed). Sets `cancellation_note`.
    pub fn cancel(&mut self, note: String) -> Result<Vec<DomainEvent>, WateringPlanError> {
        if note.trim().is_empty() {
            return Err(WateringPlanError::CancellationNoteRequired);
        }
        if !matches!(
            self.status,
            WateringPlanStatus::Planned | WateringPlanStatus::Active
        ) {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::Canceled,
            });
        }
        self.status = WateringPlanStatus::Canceled;
        self.cancellation_note = Some(note);
        Ok(vec![DomainEvent::WateringPlanCanceled {
            plan_id: self.id,
            cluster_ids: self.cluster_ids.clone(),
        }])
    }

    /// Transitions `Active → NotCompleted`.
    ///
    /// `note` must be non-empty (trimmed). Sets `cancellation_note`.
    pub fn fail(&mut self, note: String) -> Result<Vec<DomainEvent>, WateringPlanError> {
        if note.trim().is_empty() {
            return Err(WateringPlanError::CancellationNoteRequired);
        }
        if self.status != WateringPlanStatus::Active {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::NotCompleted,
            });
        }
        self.status = WateringPlanStatus::NotCompleted;
        self.cancellation_note = Some(note);
        Ok(vec![DomainEvent::WateringPlanFailed {
            plan_id: self.id,
            cluster_ids: self.cluster_ids.clone(),
        }])
    }

    /// Transitions `Active → Finished`.
    ///
    /// Requires exactly one [`WateringPlanEvaluation`] per `cluster_id` that
    /// is currently assigned. Missing evaluations result in
    /// [`WateringPlanError::EvaluationMissingForCluster`]. On success returns
    /// a [`DomainEvent::WateringPlanFinished`] with the cluster IDs and
    /// evaluations cloned for the event payload.
    pub fn finish(
        &mut self,
        evaluations: &[WateringPlanEvaluation],
    ) -> Result<Vec<DomainEvent>, WateringPlanError> {
        if self.status != WateringPlanStatus::Active {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::Finished,
            });
        }
        for cluster_id in &self.cluster_ids {
            if !evaluations.iter().any(|e| e.cluster_id == *cluster_id) {
                return Err(WateringPlanError::EvaluationMissingForCluster(*cluster_id));
            }
        }
        self.status = WateringPlanStatus::Finished;
        Ok(vec![DomainEvent::WateringPlanFinished {
            plan_id: self.id,
            cluster_ids: self.cluster_ids.clone(),
            finished_at: chrono::Utc::now(),
            evaluations: evaluations.to_vec(),
        }])
    }

    pub fn set_metrics(&mut self, metrics: RouteMetrics) {
        self.distance = metrics.distance;
        self.total_water_required = metrics.total_water_required;
        self.refill_count = metrics.refill_count;
        self.duration = metrics.duration;
        self.gpx_url = metrics.gpx_url;
        self.route_geometry = metrics.route_geometry;
        self.refill_points = metrics.refill_points;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::provenance::Provenance;
    use chrono::TimeZone;
    use claims::assert_err;

    fn fixed_plan() -> (WateringPlan, [Id<crate::cluster::TreeCluster>; 2]) {
        let c1: Id<crate::cluster::TreeCluster> = Id::new_v7();
        let c2: Id<crate::cluster::TreeCluster> = Id::new_v7();
        let plan = WateringPlan {
            id: Id::new_v7(),
            date: Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap(),
            description: None,
            start_point_name: None,
            distance: None,
            total_water_required: None,
            gpx_url: None,
            refill_count: 0,
            duration: Duration::default(),
            status: WateringPlanStatus::Planned,
            cluster_ids: vec![c1, c2],
            transporter_id: Some(Id::new_v7()),
            trailer_id: None,
            cancellation_note: None,
            provenance: Provenance::default(),
            route_geometry: None,
            refill_points: Vec::new(),
            user_ids: vec![],
        };
        (plan, [c1, c2])
    }

    #[test]
    fn start_from_planned_succeeds() {
        let (mut p, [c1, c2]) = fixed_plan();
        let events = p.start().unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Active);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::WateringPlanStarted {
                plan_id,
                cluster_ids,
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(cluster_ids, &vec![c1, c2]);
            }
            other => panic!("expected WateringPlanStarted, got {other:?}"),
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
        let (mut p, [c1, c2]) = fixed_plan();
        let events = p.cancel("not needed".to_string()).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Canceled);
        assert_eq!(p.cancellation_note(), Some("not needed"));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::WateringPlanCanceled {
                plan_id,
                cluster_ids,
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(cluster_ids, &vec![c1, c2]);
            }
            other => panic!("expected WateringPlanCanceled, got {other:?}"),
        }
    }

    #[test]
    fn cancel_from_active_succeeds() {
        let (mut p, _) = fixed_plan();
        p.start().unwrap();
        let events = p.cancel("aborted mid-run".to_string()).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Canceled);
        assert_eq!(p.cancellation_note(), Some("aborted mid-run"));
        assert!(matches!(
            events[0],
            DomainEvent::WateringPlanCanceled { .. }
        ));
    }

    #[test]
    fn cancel_from_finished_rejects() {
        let (mut p, [c1, c2]) = fixed_plan();
        p.start().unwrap();
        p.finish(&[
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: c1,
                consumed_water: 100.0,
            },
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: c2,
                consumed_water: 50.0,
            },
        ])
        .unwrap();
        assert_err!(p.cancel("too late".to_string()));
    }

    #[test]
    fn fail_only_from_active() {
        let (mut p, [c1, c2]) = fixed_plan();
        assert_err!(p.fail("breakdown".to_string()));
        p.start().unwrap();
        let events = p.fail("breakdown".to_string()).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::NotCompleted);
        assert_eq!(p.cancellation_note(), Some("breakdown"));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::WateringPlanFailed {
                plan_id,
                cluster_ids,
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(cluster_ids, &vec![c1, c2]);
            }
            other => panic!("expected WateringPlanFailed, got {other:?}"),
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
    fn finish_requires_evaluation_per_cluster() {
        let (mut p, [c1, c2]) = fixed_plan();
        p.start().unwrap();
        let only_one = vec![WateringPlanEvaluation {
            watering_plan_id: p.id,
            cluster_id: c1,
            consumed_water: 100.0,
        }];
        let err = p.finish(&only_one).unwrap_err();
        assert!(matches!(
            err,
            WateringPlanError::EvaluationMissingForCluster(id) if id == c2
        ));
    }

    #[test]
    fn finish_succeeds_emits_event_when_all_clusters_have_evaluations() {
        let (mut p, [c1, c2]) = fixed_plan();
        p.start().unwrap();
        let evals = vec![
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: c1,
                consumed_water: 100.0,
            },
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: c2,
                consumed_water: 50.0,
            },
        ];
        let events = p.finish(&evals).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Finished);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::WateringPlanFinished {
                plan_id,
                cluster_ids,
                evaluations,
                ..
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(cluster_ids, &vec![c1, c2]);
                assert_eq!(evaluations.len(), 2);
            }
            other => panic!("expected WateringPlanFinished, got {other:?}"),
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
        let result = p.replace_details(WateringPlanUpdate {
            date: p.date,
            description: None,
            start_point_name: None,
            cluster_ids: vec![],
            transporter_id: None,
            trailer_id: None,
            provenance: Provenance::default(),
            user_ids: vec![],
        });
        assert!(matches!(
            result,
            Err(WateringPlanError::CannotMutateAfterStart)
        ));
    }

    #[test]
    fn set_metrics_overwrites_run_results() {
        let (mut p, _) = fixed_plan();
        let dist = crate::shared::distance::Distance::new(1234.0).unwrap();
        let url: Url = "https://example.com/run.gpx".parse().unwrap();
        let geometry = vec![
            Coordinate::new(54.76, 9.43).unwrap(),
            Coordinate::new(54.80, 9.44).unwrap(),
        ];
        p.set_metrics(RouteMetrics {
            distance: Some(dist),
            total_water_required: Some(99.5),
            refill_count: 3,
            duration: Duration::from_secs(60 * 45),
            gpx_url: Some(url.clone()),
            route_geometry: Some(geometry.clone()),
            refill_points: Vec::new(),
        });
        assert_eq!(p.distance, Some(dist));
        assert_eq!(p.total_water_required, Some(99.5));
        assert_eq!(p.refill_count, 3);
        assert_eq!(p.duration, Duration::from_secs(60 * 45));
        assert_eq!(p.gpx_url, Some(url));
        assert_eq!(p.route_geometry(), Some(geometry.as_slice()));
    }

    #[test]
    fn set_metrics_records_and_clears_visited_refill_points() {
        let (mut p, _) = fixed_plan();
        let refill = RefillPoint {
            name: crate::start_point::StartPointName::new("Klärwerk Kielseng".to_string()).unwrap(),
            coordinate: Coordinate::new(54.8052, 9.4471).unwrap(),
        };
        p.set_metrics(RouteMetrics {
            refill_count: 1,
            duration: Duration::from_secs(60),
            refill_points: vec![refill.clone()],
            ..RouteMetrics::default()
        });
        assert_eq!(p.refill_points(), &[refill]);

        p.set_metrics(RouteMetrics::cleared());
        assert_eq!(p.refill_points(), &[]);
    }

    #[test]
    fn replace_details_sets_start_point_name() {
        let (mut p, [c1, c2]) = fixed_plan();
        p.replace_details(WateringPlanUpdate {
            date: p.date,
            description: None,
            start_point_name: Some("Betriebshof Schleswiger Straße".to_string()),
            cluster_ids: vec![c1, c2],
            transporter_id: p.transporter_id(),
            trailer_id: None,
            provenance: Provenance::default(),
            user_ids: vec![],
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
        let date = p.date;
        p.start().unwrap();
        let result = p.replace_details(WateringPlanUpdate {
            date,
            description: Some("new desc".to_string()),
            start_point_name: None,
            cluster_ids: vec![Id::new_v7()],
            transporter_id: Some(Id::new_v7()),
            trailer_id: None,
            provenance: Provenance::default(),
            user_ids: vec![],
        });
        assert!(matches!(
            result,
            Err(WateringPlanError::CannotMutateAfterStart)
        ));
    }

    #[test]
    fn revert_start_from_active_returns_to_planned() {
        let (mut p, [c1, c2]) = fixed_plan();
        p.start().unwrap();
        let events = p.revert_start().unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Planned);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::WateringPlanStartReverted {
                plan_id,
                cluster_ids,
            } => {
                assert_eq!(*plan_id, p.id);
                assert_eq!(cluster_ids, &vec![c1, c2]);
            }
            other => panic!("expected WateringPlanStartReverted, got {other:?}"),
        }
        p.start().unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Active);
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
        let (mut p, [c1, c2]) = fixed_plan();
        let u1 = uuid::Uuid::now_v7();
        let u2 = uuid::Uuid::now_v7();
        p.replace_details(WateringPlanUpdate {
            date: p.date,
            description: None,
            start_point_name: None,
            cluster_ids: vec![c1, c2],
            transporter_id: p.transporter_id(),
            trailer_id: None,
            user_ids: vec![u1, u2],
            provenance: Provenance::default(),
        })
        .unwrap();
        assert_eq!(p.user_ids(), &[u1, u2]);
    }
}
