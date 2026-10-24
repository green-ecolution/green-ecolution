use crate::{
    operation_plan::OperationPlanStatus, planning::StopRef, shared::error::ValidationError,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum OperationPlanError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("cannot go from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: OperationPlanStatus,
        to: OperationPlanStatus,
    },
    #[error("a plan can only be edited before it starts")]
    CannotMutateAfterStart,
    #[error("a cancellation needs a note")]
    CancellationNoteRequired,
    #[error("no outcome for stop {0}")]
    OutcomeMissing(StopRef),
    #[error("more than one outcome for stop {0}")]
    OutcomeDuplicate(StopRef),
    #[error("outcome for stop {0}, which is not part of the plan")]
    OutcomeUnknownStop(StopRef),
    #[error("without a trailer the transporter's own capacity must be used")]
    TransporterCapacityRequired,
}
