use thiserror::Error;

use crate::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum OrganizationError {
    #[error("the root organization cannot be modified or deleted")]
    RootImmutable,
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
