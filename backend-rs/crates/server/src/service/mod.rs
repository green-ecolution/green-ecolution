pub mod auth_service;
pub mod cluster_service;
pub mod evaluation_service;
pub mod event_bus;
pub mod handlers;
pub mod region_service;
pub mod sensor_service;
pub mod tree_service;
pub mod user_service;
pub mod vehicle_service;
pub mod watering_execution_service;
pub mod watering_plan_service;

use domain::{RepositoryError, shared::error::ValidationError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl From<ValidationError> for ServiceError {
    fn from(err: ValidationError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing or malformed authorization header")]
    MissingToken,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("forbidden: missing required role")]
    Forbidden,
    #[error("identity provider unavailable: {0}")]
    IdpUnavailable(String),
}
