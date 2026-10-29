//! What a module contributes to an operation plan: which stops exist, where
//! they are and how much capacity each one takes.

use std::fmt;

use uuid::Uuid;

use crate::{
    Id, RepositoryError, authorization::ResourceKey, capacity::CapacityKind,
    organization::Organization, shared::coordinates::Coordinate,
};

/// Opaque to the core; only the plan's module knows what it points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StopRef(Uuid);

impl StopRef {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for StopRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StopOutcome {
    pub stop: StopRef,
    pub completed: bool,
    /// Irrigation: liters used. Waste: volume loaded.
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStop {
    pub stop: StopRef,
    /// How the module names the stop to a human, e.g. on a route export.
    pub label: String,
    /// Stops without a position are skipped when routing.
    pub location: Option<Coordinate>,
    /// In liters of the provider's capacity kind.
    pub demand: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("a stop belongs to an organization outside the plan's subtree")]
    OrganizationMismatch,
    #[error("unknown stop {0}")]
    UnknownStop(StopRef),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[async_trait::async_trait]
pub trait PlanningProvider: Send + Sync {
    fn capacity_kind(&self) -> CapacityKind;

    /// Permissions on the module's plans are checked against this resource.
    fn plan_resource(&self) -> ResourceKey;

    /// Checks that every stop exists and belongs to `org` or its subtree, and
    /// returns them in the given order.
    async fn resolve_stops(
        &self,
        stops: &[StopRef],
        org: Id<Organization>,
    ) -> Result<Vec<ResolvedStop>, PlanningError>;
}
