//! Route-optimization port.
//!
//! Mirrors the `event_bus` pattern: the domain defines the trait and the
//! portable value types; a server-side adapter implements it.
//! Depot and refill locations are configuration owned by the adapter, not
//! part of this port.

use crate::{
    Id,
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

/// One cluster to visit, with its water demand in liters.
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
    pub geometry: Vec<Coordinate>,
    /// Clusters the solver could not fit into the route (logged, not fatal).
    pub unserved: Vec<Id<TreeCluster>>,
}

#[async_trait::async_trait]
pub trait RouteOptimizer: Send + Sync {
    /// Optimizes a watering route for the given vehicle, optional trailer, and stops.
    ///
    /// A depot override sets both the vehicle start and the return depot;
    /// `None` uses the adapter's configured default.
    async fn optimize(
        &self,
        transporter: &Vehicle,
        trailer: Option<&Vehicle>,
        stops: &[RouteStop],
        depot: Option<Coordinate>,
    ) -> Result<OptimizedRoute, RoutingError>;
}
