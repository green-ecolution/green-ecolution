//! Cross-aggregate evaluation summary used by the dashboard to give an
//! overview of watering activity across vehicles, regions, and entity counts.

use crate::{RepositoryError, authorization::Visibility};

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub tree_count: u32,
    pub cluster_count: u32,
    pub sensor_count: u32,
    pub watering_plan_count: i32,
    pub user_watering_plan_count: u32,
    pub vehicle_evaluation: Vec<VehicleEvaluation>,
    pub region_evaluation: Vec<RegionEvaluation>,
}

#[derive(Debug, Clone)]
pub struct VehicleEvaluation {
    pub number_plate: String,
    pub watering_plan_count: i32,
}

#[derive(Debug, Clone)]
pub struct RegionEvaluation {
    pub name: String,
    pub watering_plan_count: i32,
}

#[async_trait::async_trait]
pub trait EvaluationRepository: Send + Sync {
    /// `visible` scopes by `tree_cluster:read`, since regions carry no
    /// organization of their own and the join to `tree_clusters` is what
    /// determines whether a row may be shown.
    async fn regions_with_watering_plan(
        &self,
        visible: Visibility,
    ) -> Result<Vec<RegionEvaluation>, RepositoryError>;
    /// A row is visible only if both the vehicle (`vehicle:read`) and the
    /// watering plan (`watering_plan:read`) are visible to the caller.
    async fn vehicle_with_watering_plan(
        &self,
        visible_vehicle: Visibility,
        visible_plan: Visibility,
    ) -> Result<Vec<VehicleEvaluation>, RepositoryError>;
    /// `visible` scopes by `tree_cluster:read`, mirroring
    /// `regions_with_watering_plan` since consumption is recorded per cluster.
    async fn total_consumed_water(&self, visible: Visibility) -> Result<f64, RepositoryError>;
    /// `visible` scopes by `watering_plan:read`, since a user's assignment is
    /// only visible through the watering plan it belongs to.
    async fn watering_plan_user(&self, visible: Visibility) -> Result<u64, RepositoryError>;
}
