use crate::{Id, cluster::TreeCluster, watering_plan::WateringPlan};

/// Per-cluster outcome record submitted when finishing a watering plan.
///
/// One instance is required for every cluster assigned to the plan at the
/// time `WateringPlan::finish` is called.
#[derive(Debug, Clone, PartialEq)]
pub struct WateringPlanEvaluation {
    pub watering_plan_id: Id<WateringPlan>,
    pub cluster_id: Id<TreeCluster>,
    pub consumed_water: f64,
}
