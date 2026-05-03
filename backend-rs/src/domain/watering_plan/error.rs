use thiserror::Error;

use crate::domain::shared::error::ValidationError;
use crate::domain::watering_plan::WateringPlanStatus;

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
    #[error("evaluation missing for cluster {0}")]
    EvaluationMissingForCluster(i32),
}
