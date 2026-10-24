//! Published-event vocabulary for cross-aggregate side effects.
//!
//! Events are emitted by the service layer after aggregate mutations are
//! persisted and consumed by subscriber workers (e.g. recalculate cluster
//! centroid when a tree moves, recalculate cluster watering status when a
//! sensor is attached to or detached from a tree, or a tree's status changes).

use chrono::{DateTime, Utc};

use crate::{
    Id,
    cluster::TreeCluster,
    module::ModuleKey,
    operation_plan::OperationPlan,
    organization::Organization,
    planning::{StopOutcome, StopRef},
    role::Role,
    sensor::{
        SensorId,
        data::{VolumetricReading, Watermark},
    },
    shared::watering_status::WateringStatus,
    target::TargetRef,
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
    /// Where the sensor was attached when the reading arrived.
    pub target: Option<TargetRef>,
    pub readings: SensorReadings,
}

/// Domain events published after successful aggregate mutations.
#[derive(Debug, Clone)]
pub enum DomainEvent {
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
    /// Emitted when an activated sensor is reset back to `Prepared`. Inverse of
    /// `SensorActivated`; a released target follows as `SensorDetached`.
    SensorDeactivated {
        sensor_id: SensorId,
    },
    /// A sensor now measures for `target`.
    SensorAttached {
        sensor_id: SensorId,
        target: TargetRef,
    },
    /// A sensor stopped measuring for `target`.
    SensorDetached {
        sensor_id: SensorId,
        target: TargetRef,
    },
    /// Emitted when someone reviews a sensor's flagged readings. No subscriber
    /// today; recorded so the review is auditable alongside the other sensor
    /// lifecycle events.
    SensorDataQualityAcknowledged {
        sensor_id: SensorId,
        at: DateTime<Utc>,
        by: uuid::Uuid,
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
    OperationPlanStarted {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
    },
    OperationPlanStartReverted {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
    },
    OperationPlanCanceled {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
    },
    OperationPlanFailed {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
    },
    OperationPlanFinished {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
        finished_at: DateTime<Utc>,
        outcomes: Vec<StopOutcome>,
    },
    OperationPlanDeleted {
        plan_id: Id<OperationPlan>,
        module: ModuleKey,
        stops: Vec<StopRef>,
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
