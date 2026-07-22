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

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod auth;
pub mod authorization;
pub mod cluster;
pub mod error;
pub mod evaluation;
pub mod event_bus;
pub mod events;
pub mod info;
pub mod organization;
pub mod plugin;
pub mod region;
pub mod routing;
pub mod sensor;
pub mod sensor_model;
pub mod shared;
pub mod start_point;
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

pub type RawId = Uuid;

/// Typed UUID v7 identity.
///
/// The phantom type parameter prevents accidental cross-aggregate comparisons
/// (e.g. `Id<Tree>` cannot be compared with `Id<Vehicle>` at compile time).
/// New identifiers are generated as UUID v7 via [`Id::new_v7`]; the embedded
/// 48-bit timestamp is recoverable via [`Id::created_at`].
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

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Id<T> {
    /// Build an `Id<T>` from an existing UUID. Use this in `reconstitute`
    /// paths where the value comes from the database.
    pub fn new(id: RawId) -> Self {
        Self(id, PhantomData)
    }

    /// Generate a fresh UUID v7 identifier — the only way to mint a new id
    /// for an aggregate being persisted for the first time.
    pub fn new_v7() -> Self {
        Self(Uuid::now_v7(), PhantomData)
    }

    pub fn value(&self) -> RawId {
        self.0
    }

    /// Extract the embedded UUID v7 timestamp.
    ///
    /// Returns `None` for non-v7 UUIDs (legacy data rehydrated as v4 etc.).
    /// Used by views to populate `created_at` without a dedicated DB column.
    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        uuid_v7_timestamp(&self.0)
    }
}

/// Extract the embedded UNIX timestamp from a UUID v7.
///
/// Returns `None` for non-v7 UUIDs. Prefer [`Id::created_at`] when you already
/// have a typed `Id<T>`; this free function exists for callsites that hold a
/// raw `Uuid` (e.g. sub-entity rows like `SensorReading` whose id has no
/// aggregate type tag).
pub fn uuid_v7_timestamp(id: &Uuid) -> Option<DateTime<Utc>> {
    let (seconds, nanos) = id.get_timestamp()?.to_unix();
    DateTime::<Utc>::from_timestamp(i64::try_from(seconds).ok()?, nanos)
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
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{Id, IdSliceExt};

    #[derive(Debug)]
    struct Marker;

    #[test]
    fn id_slice_ext_extracts_inner_values() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let ids: Vec<Id<Marker>> = vec![Id::new(a), Id::new(b)];
        assert_eq!(ids.to_values(), vec![a, b]);
    }

    #[test]
    fn new_v7_produces_distinct_ids() {
        let a: Id<Marker> = Id::new_v7();
        let b: Id<Marker> = Id::new_v7();
        assert_ne!(a, b);
        assert_eq!(a.value().get_version_num(), 7);
    }

    #[test]
    fn created_at_recovers_v7_timestamp() {
        let id: Id<Marker> = Id::new_v7();
        let recovered = id.created_at().expect("v7 must encode a timestamp");
        let now = Utc::now();
        let delta = (now - recovered).num_milliseconds().abs();
        assert!(delta < 5_000, "recovered timestamp drifted by {delta} ms");
    }

    #[test]
    fn created_at_returns_none_for_v4() {
        let id: Id<Marker> = Id::new(Uuid::new_v4());
        assert!(id.created_at().is_none());
    }
}
