//! Events the irrigation module publishes about its own objects. They travel
//! the core bus wrapped in `DomainEvent::Module`, so follow-ups stay in order
//! with the core events they react to.

use domain::{
    Id,
    events::{DomainEvent, ModuleEvent},
    organization::Organization,
};

use crate::{cluster::TreeCluster, key, tree::Tree, watering_status::WateringStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum IrrigationEvent {
    TreeCreated {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    TreeDeleted {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    TreeCoordinateChanged {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    TreeMovedBetweenClusters {
        tree_id: Id<Tree>,
        from: Option<Id<TreeCluster>>,
        to: Option<Id<TreeCluster>>,
    },
    TreeWateringStatusChanged {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        new_status: WateringStatus,
    },
    /// Emitted when a cluster's tree list changes so that centroid,
    /// watering status, and region can be recalculated.
    ClusterTreesChanged { cluster_id: Id<TreeCluster> },
    /// Emitted when a cluster's `soil_condition` changes so that the
    /// volumetric watering status of member trees can be recomputed.
    ClusterSoilConditionChanged { cluster_id: Id<TreeCluster> },
    TreeResponsibilityTransferred {
        tree_id: Id<Tree>,
        from: Id<Organization>,
        to: Id<Organization>,
    },
    ClusterResponsibilityTransferred {
        cluster_id: Id<TreeCluster>,
        from: Id<Organization>,
        to: Id<Organization>,
    },
}

impl IrrigationEvent {
    pub fn into_domain(self) -> DomainEvent {
        DomainEvent::Module(ModuleEvent::new(key(), self))
    }

    /// The irrigation event inside `event`, if it is one.
    pub fn from_domain(event: &DomainEvent) -> Option<&IrrigationEvent> {
        match event {
            DomainEvent::Module(inner) => inner.downcast_ref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_the_core_bus_type() {
        let cluster_id = Id::new_v7();
        let event = IrrigationEvent::ClusterTreesChanged { cluster_id }.into_domain();
        assert_eq!(
            IrrigationEvent::from_domain(&event),
            Some(&IrrigationEvent::ClusterTreesChanged { cluster_id })
        );
        let core = DomainEvent::OrganizationDeleted {
            organization_id: Id::new_v7(),
        };
        assert!(IrrigationEvent::from_domain(&core).is_none());
    }
}
