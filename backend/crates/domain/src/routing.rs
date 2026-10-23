//! Route-optimization port.
//!
//! Mirrors the `event_bus` pattern: the domain defines the trait and the
//! portable value types; a server-side adapter implements it.

use crate::{
    Id,
    cluster::TreeCluster,
    shared::{coordinates::Coordinate, distance::Distance, water_capacity::WaterCapacity},
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
    /// Indices into the caller-supplied refill station list of the stations
    /// the route actually visits, deduplicated in first-visit order.
    pub refill_station_indices: Vec<usize>,
    pub geometry: Vec<Coordinate>,
    /// Clusters the solver could not fit into the route (logged, not fatal).
    pub unserved: Vec<Id<TreeCluster>>,
}

/// Water a route can carry before it has to refill.
pub fn tank_capacity(
    transporter: &Vehicle,
    trailer: Option<&Vehicle>,
    transporter_tank_used: bool,
) -> WaterCapacity {
    let empty = WaterCapacity::new(0.0).expect("zero litres is non-negative");
    let own = if transporter_tank_used {
        transporter.water_capacity
    } else {
        empty
    };
    own + trailer.map_or(empty, |t| t.water_capacity)
}

#[async_trait::async_trait]
pub trait RouteOptimizer: Send + Sync {
    /// Optimizes a watering route for the given vehicle and stops.
    /// `capacity` is the water the run carries (see [`tank_capacity`]); the
    /// transporter only contributes its dimensions. `depot` is the resolved
    /// start/return location and `refill_stations` are the water refill
    /// points, both supplied by the caller (loaded from persistence), not
    /// owned by the adapter.
    async fn optimize(
        &self,
        transporter: &Vehicle,
        capacity: WaterCapacity,
        stops: &[RouteStop],
        depot: Coordinate,
        refill_stations: &[Coordinate],
    ) -> Result<OptimizedRoute, RoutingError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vehicle::{DrivingLicense, VehicleAvailability, VehicleSnapshot, VehicleType};

    fn vehicle(vehicle_type: VehicleType, water_capacity: f64) -> Vehicle {
        Vehicle::reconstitute(VehicleSnapshot {
            id: uuid::Uuid::now_v7(),
            archived_at: None,
            number_plate: "FL GE 1234".to_string(),
            description: None,
            water_capacity,
            availability: VehicleAvailability::Available,
            vehicle_type,
            model: "Unimog".to_string(),
            driving_license: DrivingLicense::BE,
            height: 2.1,
            width: 2.0,
            length: 5.0,
            weight: 3500.0,
            provider: None,
            additional_info: None,
            organization_id: uuid::Uuid::now_v7(),
        })
    }

    #[test]
    fn tank_capacity_adds_trailer_to_transporter() {
        let transporter = vehicle(VehicleType::Transporter, 1000.0);
        let trailer = vehicle(VehicleType::Trailer, 500.0);
        assert_eq!(
            tank_capacity(&transporter, Some(&trailer), true).liters(),
            1500.0
        );
    }

    #[test]
    fn tank_capacity_skips_unused_transporter_tank() {
        let transporter = vehicle(VehicleType::Transporter, 1000.0);
        let trailer = vehicle(VehicleType::Trailer, 500.0);
        assert_eq!(
            tank_capacity(&transporter, Some(&trailer), false).liters(),
            500.0
        );
    }

    #[test]
    fn tank_capacity_without_trailer_is_transporter_tank() {
        let transporter = vehicle(VehicleType::Transporter, 1000.0);
        assert_eq!(tank_capacity(&transporter, None, true).liters(), 1000.0);
    }
}
