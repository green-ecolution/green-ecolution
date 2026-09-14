use thiserror::Error;

use crate::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum PluginError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
