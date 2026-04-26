use crate::domain::RepositoryError;

pub mod pg_cluster;
pub mod pg_evaluation;
pub mod system_info;
pub mod pg_region;
pub mod pg_sensor;
pub mod pg_tree;
pub mod pg_vehicle;
pub mod pg_watering_plan;

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
