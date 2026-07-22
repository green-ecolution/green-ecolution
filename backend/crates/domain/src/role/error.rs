use thiserror::Error;

use crate::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum RoleError {
    #[error("template roles cannot be modified or deleted; copy them into an organization instead")]
    TemplateImmutable,
    #[error("template roles cannot be assigned to users")]
    CannotAssignTemplate,
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
