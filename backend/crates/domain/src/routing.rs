//! Route-optimization port.
//!
//! Mirrors the `event_bus` pattern: the domain defines the trait and the
//! portable value types; a server-side adapter implements it.

use crate::{
    capacity::{CapacityKind, Liters},
    planning::StopRef,
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
    #[error("no station replenishes {kind}")]
    NoStation { kind: CapacityKind },
}

/// One stop to visit, with its demand in liters of the route's capacity kind.
#[derive(Debug, Clone)]
pub struct RouteStop {
    pub stop: StopRef,
    pub location: Coordinate,
    pub demand: f64,
}

/// One visit of a solved route. `Station` indexes the caller-supplied
/// station list of the [`RouteRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteVisit {
    Depot,
    Stop(StopRef),
    Station(usize),
}

#[derive(Debug, Clone)]
pub struct OptimizedRoute {
    pub distance: Distance,
    pub duration: std::time::Duration,
    pub refill_count: u32,
    /// Every visit in driving order, repeat station visits included.
    pub visits: Vec<RouteVisit>,
    pub geometry: Vec<Coordinate>,
    /// Stops the solver could not fit into the route (logged, not fatal).
    pub unserved: Vec<StopRef>,
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
