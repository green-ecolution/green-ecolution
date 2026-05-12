use axum::{http::StatusCode, response::IntoResponse};

use crate::service::{AuthError, ServiceError};
use domain::RepositoryError;

fn repository_error_status(e: &RepositoryError) -> StatusCode {
    match e {
        RepositoryError::NotFound => StatusCode::NOT_FOUND,
        RepositoryError::AlreadyExists(_) => StatusCode::CONFLICT,
        RepositoryError::ForeignKeyViolation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        RepositoryError::ConstraintViolation(_) => StatusCode::BAD_REQUEST,
        RepositoryError::DataIntegrity(_) | RepositoryError::Internal(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AuthError::MissingToken | AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            AuthError::IdpUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        if status.is_server_error() {
            tracing::error!(error = %self, kind = "auth", "request failed");
        }
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServiceError::Repository(e) => {
                let status = repository_error_status(&e);
                if status.is_server_error() {
                    tracing::error!(error = %e, kind = "repository", "request failed");
                }
                (status, e.to_string()).into_response()
            }
            ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ServiceError::Auth(e) => e.into_response(),
            e @ (ServiceError::TreeAlreadyHasSensor | ServiceError::AlreadyActivated) => {
                (StatusCode::CONFLICT, e.to_string()).into_response()
            }
        }
    }
}
