//! Published-event vocabulary for cross-aggregate side effects.
//!
//! Events are emitted by the service layer after aggregate mutations are
//! persisted and consumed by subscriber workers (e.g. recalculate cluster
//! centroid when a tree moves, detach sensor from trees when a sensor is
//! deleted).

use crate::domain::{
    Id,
    cluster::TreeCluster,
    sensor::SensorId,
    tree::Tree,
    watering_plan::{WateringPlan, WateringPlanStatus},
};

/// Domain events published after successful aggregate mutations.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    TreeCreated {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    TreeUpdated {
        tree_id: Id<Tree>,
        /// Previous cluster assignment, used to recalculate the old cluster.
        old_cluster_id: Option<Id<TreeCluster>>,
        new_cluster_id: Option<Id<TreeCluster>>,
    },
    TreeDeleted {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    /// Emitted when a cluster's tree list changes so that centroid,
    /// watering status, and region can be recalculated.
    ClusterTreesChanged { cluster_id: Id<TreeCluster> },
    /// Emitted when a sensor is deleted; subscribers detach it from
    /// `affected_tree_ids` and reset their watering status.
    SensorDeleted {
        sensor_id: SensorId,
        affected_tree_ids: Vec<Id<Tree>>,
    },
    SensorDataReceived {
        sensor_id: SensorId,
        data: serde_json::Value,
    },
    WateringPlanStatusChanged {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
        new_status: WateringPlanStatus,
    },
}
