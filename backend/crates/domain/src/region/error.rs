use thiserror::Error;

use crate::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum RegionError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
