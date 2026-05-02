use thiserror::Error;

use crate::domain::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum TreeError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
