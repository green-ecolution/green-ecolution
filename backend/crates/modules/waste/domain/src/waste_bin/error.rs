use thiserror::Error;

use domain::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum WasteBinError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
