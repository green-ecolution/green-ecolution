//! WateringPlan aggregate — a scheduled watering run for a set of tree clusters.
//!
//! ## State machine
//!
//! ```text
//! Planned ──start()──► Active ──finish(evals)──► Finished
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

use crate::domain::{
    Id,
    cluster::TreeCluster,
    events::DomainEvent,
    shared::{
        distance::Distance,
        provenance::{Provenance, ProviderId},
    },
    vehicle::Vehicle,
};

pub use error::WateringPlanError;
pub use error::WateringPlanError as Error;
pub use evaluation::WateringPlanEvaluation;
pub use repository::{WateringPlanReader, WateringPlanWriter};
#[allow(unused_imports)]
pub(crate) use snapshot::WateringPlanSnapshot;
pub use view::WateringPlanView;

/// Lifecycle status of a [`WateringPlan`].
///
/// See the module-level state machine diagram for valid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "watering_plan_status", rename_all = "snake_case")]
pub enum WateringPlanStatus {
    Planned,
    Active,
    Canceled,
    Finished,
    #[sqlx(rename = "not competed")]
    NotCompleted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WateringPlan {
    pub id: Id<WateringPlan>,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
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
}

/// Input for creating a new [`WateringPlan`].
#[derive(Debug, Clone)]
pub struct WateringPlanDraft {
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub provenance: Provenance,
}

#[derive(Debug, Default, Clone)]
pub struct WateringPlanSearchQuery {
    pub provider: Option<ProviderId>,
}

/// Replacement input for [`WateringPlan`] field edits while still
/// in [`WateringPlanStatus::Planned`].
#[derive(Debug, Clone)]
pub struct WateringPlanUpdate {
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub provenance: Provenance,
}

impl WateringPlan {
    #[allow(dead_code)]
    pub(crate) fn reconstitute(snap: WateringPlanSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            date: snap.date,
            description: snap.description,
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
        }
    }

    pub fn status(&self) -> WateringPlanStatus {
        self.status
    }

    pub fn cluster_ids(&self) -> &[Id<TreeCluster>] {
        &self.cluster_ids
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

    fn ensure_planned(&self) -> Result<(), WateringPlanError> {
        if self.status != WateringPlanStatus::Planned {
            return Err(WateringPlanError::CannotMutateAfterStart);
        }
        Ok(())
    }

    /// Updates editable fields. Only allowed while status is [`WateringPlanStatus::Planned`].
    pub fn replace_details(&mut self, update: WateringPlanUpdate) -> Result<(), WateringPlanError> {
        self.ensure_planned()?;
        self.date = update.date;
        self.description = update.description;
        self.cluster_ids = update.cluster_ids;
        self.transporter_id = update.transporter_id;
        self.trailer_id = update.trailer_id;
        self.provenance = update.provenance;
        Ok(())
    }

    /// Transitions `Planned → Active`. Fails on any other starting status.
    pub fn start(&mut self) -> Result<(), WateringPlanError> {
        if self.status != WateringPlanStatus::Planned {
            return Err(WateringPlanError::InvalidStateTransition {
                from: self.status,
                to: WateringPlanStatus::Active,
            });
        }
        self.status = WateringPlanStatus::Active;
        Ok(())
    }

    /// Transitions `Planned | Active → Canceled`.
    ///
    /// `note` must be non-empty (trimmed). Sets `cancellation_note`.
    pub fn cancel(&mut self, note: String) -> Result<(), WateringPlanError> {
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
        Ok(())
    }

    /// Transitions `Active → NotCompleted`.
    ///
    /// `note` must be non-empty (trimmed). Sets `cancellation_note`.
    pub fn fail(&mut self, note: String) -> Result<(), WateringPlanError> {
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
        Ok(())
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
                return Err(WateringPlanError::EvaluationMissingForCluster(
                    cluster_id.value(),
                ));
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

    #[allow(dead_code)]
    pub fn set_metrics(
        &mut self,
        distance: Option<Distance>,
        total_water_required: Option<f64>,
        refill_count: u32,
        duration: Duration,
        gpx_url: Option<Url>,
    ) {
        self.distance = distance;
        self.total_water_required = total_water_required;
        self.refill_count = refill_count;
        self.duration = duration;
        self.gpx_url = gpx_url;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::provenance::Provenance;
    use chrono::TimeZone;
    use claims::{assert_err, assert_ok};

    fn fixed_plan() -> WateringPlan {
        WateringPlan {
            id: Id::new(1),
            date: Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap(),
            description: None,
            distance: None,
            total_water_required: None,
            gpx_url: None,
            refill_count: 0,
            duration: Duration::default(),
            status: WateringPlanStatus::Planned,
            cluster_ids: vec![Id::new(1), Id::new(2)],
            transporter_id: Some(Id::new(10)),
            trailer_id: None,
            cancellation_note: None,
            provenance: Provenance::default(),
        }
    }

    #[test]
    fn start_from_planned_succeeds() {
        let mut p = fixed_plan();
        assert_ok!(p.start());
        assert_eq!(p.status(), WateringPlanStatus::Active);
    }

    #[test]
    fn start_from_active_rejects() {
        let mut p = fixed_plan();
        p.start().unwrap();
        assert_err!(p.start());
    }

    #[test]
    fn cancel_requires_note() {
        let mut p = fixed_plan();
        assert_err!(p.cancel("".to_string()));
        assert_err!(p.cancel("   ".to_string()));
    }

    #[test]
    fn cancel_from_planned_succeeds() {
        let mut p = fixed_plan();
        p.cancel("not needed".to_string()).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Canceled);
        assert_eq!(p.cancellation_note(), Some("not needed"));
    }

    #[test]
    fn cancel_from_finished_rejects() {
        let mut p = fixed_plan();
        p.start().unwrap();
        p.finish(&[
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: Id::new(1),
                consumed_water: 100.0,
            },
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: Id::new(2),
                consumed_water: 50.0,
            },
        ])
        .unwrap();
        assert_err!(p.cancel("too late".to_string()));
    }

    #[test]
    fn fail_only_from_active() {
        let mut p = fixed_plan();
        assert_err!(p.fail("breakdown".to_string()));
        p.start().unwrap();
        assert_ok!(p.fail("breakdown".to_string()));
        assert_eq!(p.status(), WateringPlanStatus::NotCompleted);
    }

    #[test]
    fn finish_requires_active() {
        let mut p = fixed_plan();
        assert_err!(p.finish(&[]));
    }

    #[test]
    fn finish_requires_evaluation_per_cluster() {
        let mut p = fixed_plan();
        p.start().unwrap();
        let only_one = vec![WateringPlanEvaluation {
            watering_plan_id: p.id,
            cluster_id: Id::new(1),
            consumed_water: 100.0,
        }];
        let err = p.finish(&only_one).unwrap_err();
        assert!(matches!(
            err,
            WateringPlanError::EvaluationMissingForCluster(2)
        ));
    }

    #[test]
    fn finish_succeeds_emits_event_when_all_clusters_have_evaluations() {
        let mut p = fixed_plan();
        p.start().unwrap();
        let evals = vec![
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: Id::new(1),
                consumed_water: 100.0,
            },
            WateringPlanEvaluation {
                watering_plan_id: p.id,
                cluster_id: Id::new(2),
                consumed_water: 50.0,
            },
        ];
        let events = p.finish(&evals).unwrap();
        assert_eq!(p.status(), WateringPlanStatus::Finished);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::WateringPlanFinished { .. }
        ));
    }

    #[test]
    fn replace_details_only_when_planned() {
        let mut p = fixed_plan();
        let date = p.date;
        p.start().unwrap();
        let result = p.replace_details(WateringPlanUpdate {
            date,
            description: Some("new desc".to_string()),
            cluster_ids: vec![Id::new(3)],
            transporter_id: Some(Id::new(11)),
            trailer_id: None,
            provenance: Provenance::default(),
        });
        assert!(matches!(
            result,
            Err(WateringPlanError::CannotMutateAfterStart)
        ));
    }
}
