use crate::domain::RepositoryError;

pub mod pg_region;

impl From<sqlx::Error> for RepositoryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                Some("23505") => RepositoryError::AlreadyExists(db_err.to_string()),
                Some("23503") => RepositoryError::ForeignKeyViolation(db_err.to_string()),
                _ => RepositoryError::Internal(db_err.to_string()),
            },
            _ => RepositoryError::Internal(value.to_string()),
        }
    }
}
