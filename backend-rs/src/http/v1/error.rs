use axum::{http::StatusCode, response::IntoResponse};

use crate::{domain::RepositoryError, service::ServiceError};

impl IntoResponse for RepositoryError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            RepositoryError::NotFound => StatusCode::NOT_FOUND,
            RepositoryError::AlreadyExists(_) => StatusCode::CONFLICT,
            RepositoryError::ForeignKeyViolation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            RepositoryError::ConstraintViolation(_) => StatusCode::BAD_REQUEST,
            RepositoryError::DataIntegrity(_) | RepositoryError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServiceError::Repository(e) => e.into_response(),
            ServiceError::Domain(e) => {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
            ServiceError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, msg).into_response()
            }
        }
    }
}
