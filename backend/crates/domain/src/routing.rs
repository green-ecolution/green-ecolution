//! Route-optimization port.
//!
//! Mirrors the `event_bus` pattern: the domain defines the trait and the
//! portable value types; a server-side adapter implements it.

use crate::{
    Id,
    capacity::{CapacityKind, Liters},
    cluster::TreeCluster,
    shared::{coordinates::Coordinate, distance::Distance},
    vehicle::Vehicle,
};

#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("routing engine unavailable: {0}")]
    Unavailable(String),
    #[error("route problem rejected: {0}")]
    InvalidProblem(String),
    #[error("route optimization failed: {0}")]
    Failed(String),
}

/// One cluster to visit, with its demand in liters.
#[derive(Debug, Clone)]
pub struct RouteStop {
    pub cluster_id: Id<TreeCluster>,
    pub location: Coordinate,
    pub demand_liters: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizedRoute {
    pub distance: Distance,
    pub duration: std::time::Duration,
    pub refill_count: u32,
    /// Indices into the caller-supplied refill station list of the stations
    /// the route actually visits, deduplicated in first-visit order.
    pub refill_station_indices: Vec<usize>,
    pub geometry: Vec<Coordinate>,
    /// Clusters the solver could not fit into the route (logged, not fatal).
    pub unserved: Vec<Id<TreeCluster>>,
}

#[derive(Debug, Clone)]
pub struct RouteRequest<'a> {
    pub transporter: &'a Vehicle,
    pub kind: CapacityKind,
    pub capacity: Liters,
    pub stops: &'a [RouteStop],
    pub depot: Coordinate,
    /// Only stations that restore `kind`.
    pub stations: &'a [Coordinate],
    /// The route must visit a station before returning, e.g. to unload.
    pub return_replenished: bool,
}

#[async_trait::async_trait]
pub trait RouteOptimizer: Send + Sync {
    async fn optimize(&self, request: RouteRequest<'_>) -> Result<OptimizedRoute, RoutingError>;
}
