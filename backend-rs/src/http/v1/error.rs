use axum::{http::StatusCode, response::IntoResponse};

use crate::domain::RepositoryError;

impl IntoResponse for RepositoryError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            RepositoryError::NotFound => StatusCode::NOT_FOUND,
            RepositoryError::AlreadyExists(_) => StatusCode::CONFLICT,
            RepositoryError::ForeignKeyViolation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            RepositoryError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
