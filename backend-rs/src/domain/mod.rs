use std::marker::PhantomData;

pub mod auth;
pub mod cluster;
pub mod evaluation;
pub mod events;
pub mod info;
pub mod plugin;
pub mod region;
pub mod sensor;
pub mod shared;
pub mod tree;
pub mod user;
pub mod vehicle;
pub mod watering_plan;

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

impl From<crate::domain::shared::error::ValidationError> for RepositoryError {
    fn from(e: crate::domain::shared::error::ValidationError) -> Self {
        RepositoryError::DataIntegrity(e.to_string())
    }
}

#[derive(Debug)]
pub struct Id<T>(i32, PhantomData<T>);

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> From<i32> for Id<T> {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Id<T> {
    pub fn new(id: i32) -> Self {
        Self(id, PhantomData)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}
