use thiserror::Error;

use crate::domain::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum VehicleError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
