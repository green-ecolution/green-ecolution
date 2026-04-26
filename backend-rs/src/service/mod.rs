pub mod cluster_service;
pub mod evaluation_service;
pub mod event_bus;
pub mod handlers;
pub mod region_service;
pub mod sensor_service;
pub mod tree_service;
pub mod vehicle_service;
pub mod watering_plan_service;

use crate::domain::{DomainError, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
