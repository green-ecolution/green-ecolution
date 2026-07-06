//! Conversions into [`crate::RepositoryError`].
//!
//! Driver-specific errors get mapped here so the domain layer never depends
//! directly on a particular DB or auth library.

use crate::RepositoryError;
use crate::shared::error::ValidationError;

impl From<ValidationError> for RepositoryError {
    fn from(e: ValidationError) -> Self {
        RepositoryError::DataIntegrity(e.to_string())
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for RepositoryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                Some("23505") => RepositoryError::AlreadyExists(db_err.to_string()),
                Some("23503") => RepositoryError::ForeignKeyViolation(db_err.to_string()),
                Some("23502") => RepositoryError::ConstraintViolation(db_err.to_string()),
                Some("23514") => RepositoryError::ConstraintViolation(db_err.to_string()),
                _ => RepositoryError::Internal(db_err.to_string()),
            },
            _ => RepositoryError::Internal(value.to_string()),
        }
    }
}
