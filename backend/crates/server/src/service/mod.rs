pub mod authorization;
pub mod cluster_service;
pub mod evaluation_service;
pub mod event_bus;
pub mod handlers;
pub mod organization_service;
pub mod region_service;
pub mod role_service;
pub mod sensor_service;
pub mod start_point_service;
pub mod tree_service;
pub mod user_service;
pub mod vehicle_service;
pub mod watering_execution_service;
pub mod watering_plan_service;

use domain::{RepositoryError, routing::RoutingError, shared::error::ValidationError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("tree already has a different sensor")]
    TreeAlreadyHasSensor,
    #[error("sensor is already assigned to another tree")]
    SensorAlreadyAssigned,
    #[error("sensor is already activated")]
    AlreadyActivated,
    #[error("sensor is not activated")]
    NotActivated,
    #[error("{feature} feature is disabled")]
    FeatureDisabled { feature: &'static str },
    #[error(transparent)]
    Routing(#[from] RoutingError),
    #[error(transparent)]
    Organization(#[from] domain::organization::OrganizationError),
    #[error(transparent)]
    Role(#[from] domain::role::RoleError),
    #[error("organization still has sub-organizations or users")]
    OrganizationNotEmpty,
    #[error("resource and organization do not match")]
    OrganizationMismatch,
    #[error("tree is part of a cluster")]
    TreeInCluster,
    #[error("share target must be a proper descendant of the owning organization")]
    ShareTargetNotDescendant,
    #[error("no organization given and the acting user has none")]
    MissingOrganization,
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
    #[error("forbidden: missing required permission")]
    Forbidden,
    #[error("identity provider unavailable: {0}")]
    IdpUnavailable(String),
}
