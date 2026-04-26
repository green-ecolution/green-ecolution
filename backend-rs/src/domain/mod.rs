use std::marker::PhantomData;

pub mod auth;
pub mod cluster;
pub mod evaluation;
pub mod info;
pub mod plugin;
pub mod region;
pub mod sensor;
pub mod shared;
pub mod tree;
pub mod vehicle;
pub mod watering_plan;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("planting year must be > 0 and not in the future, got {0}")]
    InvalidPlantingYear(u32),
    #[error("latitude must be between -90 and 90")]
    InvalidLatitude(f64),
    #[error("longitude must be between -180 and 180")]
    InvalidLongitude(f64),
    #[error("water capacity must be >= 0")]
    InvalidWaterCapacity(f64),
    #[error("distance must be >= 0")]
    InvalidDistance(f64),
    #[error("invalid driving license")]
    InvalidDrivingLicense(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,
    #[error("entity already exists: {0}")]
    AlreadyExists(String),
    #[error("referenced entity not found: {0}")]
    ForeignKeyViolation(String),
    #[error("constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("data integrity error: {0}")]
    DataIntegrity(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<DomainError> for RepositoryError {
    fn from(value: DomainError) -> Self {
        RepositoryError::DataIntegrity(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T>(i32, PhantomData<T>);

impl<T> Id<T> {
    pub fn new(id: i32) -> Self {
        Self(id, PhantomData)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}
