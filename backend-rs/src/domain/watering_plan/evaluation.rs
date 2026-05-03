use crate::domain::{Id, cluster::TreeCluster, watering_plan::WateringPlan};

#[derive(Debug, Clone, PartialEq)]
pub struct WateringPlanEvaluation {
    pub watering_plan_id: Id<WateringPlan>,
    pub cluster_id: Id<TreeCluster>,
    pub consumed_water: f64,
}
