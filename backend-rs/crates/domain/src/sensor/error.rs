use thiserror::Error;

use crate::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum SensorError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("sensor is already activated")]
    AlreadyActivated,
    #[error("sensor is not activated")]
    NotActivated,
}
