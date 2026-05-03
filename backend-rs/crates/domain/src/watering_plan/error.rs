use thiserror::Error;

use crate::shared::error::ValidationError;
use crate::watering_plan::WateringPlanStatus;

/// Errors from `WateringPlan` state transitions and validation.
#[derive(Debug, Error, PartialEq)]
pub enum WateringPlanError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("invalid status transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: WateringPlanStatus,
        to: WateringPlanStatus,
    },
    #[error("plan content can only be modified while planned")]
    CannotMutateAfterStart,
    #[error("a non-empty cancellation note is required")]
    CancellationNoteRequired,
    /// Raised by `WateringPlan::finish` when no evaluation is present for the
    /// given `cluster_id`. Every cluster currently assigned to the plan must
    /// have exactly one evaluation entry.
    #[error("evaluation missing for cluster {0}")]
    EvaluationMissingForCluster(i32),
}
