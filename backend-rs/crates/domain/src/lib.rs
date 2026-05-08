//! Domain layer for Green Ecolution.
//!
//! Each aggregate lives in its own sub-module and follows a consistent pattern:
//!
//! - **Aggregate** (`Tree`, `Vehicle`, …) — enforces invariants; key fields
//!   that must not be set directly are private with accessor methods.
//! - **View** (`TreeView`, …) — flat, audit-enriched read model (`created_at` /
//!   `updated_at`) used by HTTP handlers; avoids exposing aggregate internals.
//! - **Snapshot** (`pub(crate)`) — raw DB-row struct used exclusively for
//!   rehydration via `reconstitute`; never crosses the domain boundary.
//! - **Reader / Writer traits** — split so read-heavy paths depend only on
//!   `*Reader` and mutation paths only on `*Writer`.
//! - **Draft** — plain input struct for aggregate creation (`save_new`).
//!
//! [`shared`] holds cross-cutting value objects and `ValidationError`.
//! [`RepositoryError`] is the single error type returned by all repository
//! traits; `ValidationError` converts into it via `DataIntegrity`.

use std::marker::PhantomData;

pub mod auth;
pub mod cluster;
pub mod error;
pub mod evaluation;
pub mod event_bus;
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

/// Errors that repository implementations return to the domain.
///
/// Infrastructure adapters map driver-specific errors into these variants so
/// that the domain layer never depends on a particular DB or auth library.
/// `ValidationError` converts into [`RepositoryError::DataIntegrity`].
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

pub type RawId = i32;

/// Typed integer identity.
///
/// The phantom type parameter prevents accidental cross-aggregate comparisons
/// (e.g. `Id<Tree>` cannot be compared with `Id<Vehicle>` at compile time).
#[derive(Debug)]
pub struct Id<T>(RawId, PhantomData<T>);

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

impl<T> From<RawId> for Id<T> {
    fn from(value: RawId) -> Self {
        Self::new(value)
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Id<T> {
    pub fn new(id: RawId) -> Self {
        Self(id, PhantomData)
    }

    pub fn value(&self) -> RawId {
        self.0
    }
}

pub trait IdSliceExt {
    fn to_values(&self) -> Vec<RawId>;
}

impl<T> IdSliceExt for [Id<T>] {
    fn to_values(&self) -> Vec<RawId> {
        self.iter().map(Id::value).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Id, IdSliceExt};

    #[test]
    fn id_slice_ext_extracts_inner_values() {
        struct Marker;
        let ids: Vec<Id<Marker>> = vec![Id::new(1), Id::new(2), Id::new(3)];
        assert_eq!(ids.to_values(), vec![1, 2, 3]);
    }
}
