use crate::domain::{
    Id,
    cluster::TreeCluster,
    sensor::SensorId,
    tree::Tree,
    watering_plan::{WateringPlan, WateringPlanStatus},
};

#[derive(Debug, Clone)]
pub enum DomainEvent {
    TreeCreated {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    TreeUpdated {
        tree_id: Id<Tree>,
        old_cluster_id: Option<Id<TreeCluster>>,
        new_cluster_id: Option<Id<TreeCluster>>,
    },
    TreeDeleted {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
    },
    ClusterTreesChanged {
        cluster_id: Id<TreeCluster>,
    },
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
