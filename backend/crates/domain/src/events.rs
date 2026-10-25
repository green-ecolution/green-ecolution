//! Published-event vocabulary for cross-aggregate side effects.
//!
//! Events are emitted by the service layer after aggregate mutations are
//! persisted and consumed by subscriber workers (e.g. recalculate cluster
//! centroid when a tree moves, recalculate cluster watering status when a
//! sensor is attached to or detached from a tree, or a tree's status changes).

use std::{any::Any, fmt, sync::Arc};

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
};

/// Payload of an event a module defines for itself. The core never looks
/// inside; the module's own handlers recover it with `downcast_ref`.
pub trait ModuleEventPayload: Any + fmt::Debug + Send + Sync {}

impl<T: Any + fmt::Debug + Send + Sync> ModuleEventPayload for T {}

#[derive(Debug, Clone)]
pub struct ModuleEvent {
    module: ModuleKey,
    payload: Arc<dyn ModuleEventPayload>,
}

impl ModuleEvent {
    pub fn new<P: ModuleEventPayload>(module: ModuleKey, payload: P) -> Self {
        Self {
            module,
            payload: Arc::new(payload),
        }
    }

    pub fn module(&self) -> &ModuleKey {
        &self.module
    }

    pub fn downcast_ref<P: ModuleEventPayload>(&self) -> Option<&P> {
        (self.payload.as_ref() as &dyn Any).downcast_ref()
    }
}

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
    /// An event a module defined for its own objects, e.g. a tree moving
    /// between clusters. Travels the same bus so follow-ups keep their order.
    Module(ModuleEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Probe {
        Ping(u8),
    }

    #[derive(Debug)]
    struct Other;

    fn key() -> ModuleKey {
        ModuleKey::new("probe").unwrap()
    }

    #[test]
    fn downcasts_to_its_own_payload_only() {
        let event = ModuleEvent::new(key(), Probe::Ping(7));
        assert_eq!(event.module().as_str(), "probe");
        assert_eq!(event.downcast_ref::<Probe>(), Some(&Probe::Ping(7)));
        assert!(event.downcast_ref::<Other>().is_none());
    }

    #[test]
    fn clones_share_the_payload() {
        let event = DomainEvent::Module(ModuleEvent::new(key(), Probe::Ping(1)));
        let copy = event.clone();
        let DomainEvent::Module(inner) = copy else {
            panic!("expected a module event")
        };
        assert_eq!(inner.downcast_ref::<Probe>(), Some(&Probe::Ping(1)));
    }
}
