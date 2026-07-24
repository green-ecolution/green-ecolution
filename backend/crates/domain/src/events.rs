//! Published-event vocabulary for cross-aggregate side effects.
//!
//! Events are emitted by the service layer after aggregate mutations are
//! persisted and consumed by subscriber workers (e.g. recalculate cluster
//! centroid when a tree moves, recalculate cluster watering status when a
//! tree's sensor link or status changes).

use chrono::{DateTime, Utc};

use crate::{
    Id,
    cluster::TreeCluster,
    organization::Organization,
    role::Role,
    sensor::{
        SensorId,
        data::{VolumetricReading, Watermark},
    },
    shared::watering_status::WateringStatus,
    tree::Tree,
    watering_plan::{WateringPlan, WateringPlanEvaluation},
};

#[derive(Debug, Clone)]
pub enum SensorReadings {
    Watermarks(Vec<Watermark>),
    Volumetrics(Vec<VolumetricReading>),
}

#[derive(Debug, Clone)]
pub struct SensorDataReceivedPayload {
    pub sensor_id: SensorId,
    pub readings: SensorReadings,
}

/// Domain events published after successful aggregate mutations.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    TreeCreated {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        sensor_id: Option<SensorId>,
    },
    TreeDeleted {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        had_sensor: bool,
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
    TreeSensorAttached {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        sensor_id: SensorId,
    },
    TreeSensorDetached {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        sensor_id: SensorId,
    },
    TreeWateringStatusChanged {
        tree_id: Id<Tree>,
        cluster_id: Option<Id<TreeCluster>>,
        new_status: WateringStatus,
    },
    /// Emitted when a cluster's tree list changes so that centroid,
    /// watering status, and region can be recalculated.
    ClusterTreesChanged {
        cluster_id: Id<TreeCluster>,
    },
    /// Emitted when a cluster's `soil_condition` changes so that the
    /// volumetric watering status of member trees can be recomputed.
    ClusterSoilConditionChanged {
        cluster_id: Id<TreeCluster>,
    },
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
    SensorResponsibilityTransferred {
        sensor_id: SensorId,
        from: Id<Organization>,
        to: Id<Organization>,
    },
    /// Emitted after a sensor reading is persisted. Carries the parsed
    /// readings so subscribers don't have to re-parse the raw JSON payload.
    SensorDataReceived(SensorDataReceivedPayload),
    /// Emitted when a sensor transitions from `Prepared` to `Offline`.
    SensorActivated {
        sensor_id: SensorId,
    },
    /// Emitted when an activated sensor is reset back to `Prepared`,
    /// dropping its tree link. Inverse of `SensorActivated`.
    SensorDeactivated {
        sensor_id: SensorId,
    },
    WateringPlanStarted {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
    },
    WateringPlanStartReverted {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
    },
    WateringPlanCanceled {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
    },
    WateringPlanFailed {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
    },
    WateringPlanFinished {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
        finished_at: DateTime<Utc>,
        evaluations: Vec<WateringPlanEvaluation>,
    },
    WateringPlanDeleted {
        plan_id: Id<WateringPlan>,
        cluster_ids: Vec<Id<TreeCluster>>,
    },
    OrganizationCreated {
        organization_id: Id<Organization>,
    },
    OrganizationRenamed {
        organization_id: Id<Organization>,
    },
    OrganizationDeleted {
        organization_id: Id<Organization>,
    },
    RoleCreated {
        role_id: Id<Role>,
        organization_id: Id<Organization>,
    },
    RoleRenamed {
        role_id: Id<Role>,
    },
    RolePermissionsChanged {
        role_id: Id<Role>,
    },
    RoleDeleted {
        role_id: Id<Role>,
    },
}
