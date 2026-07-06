//! Cross-aggregate evaluation summary used by the dashboard to give an
//! overview of watering activity across vehicles, regions, and entity counts.

use crate::RepositoryError;

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
    async fn regions_with_watering_plan(&self) -> Result<Vec<RegionEvaluation>, RepositoryError>;
    async fn vehicle_with_watering_plan(&self) -> Result<Vec<VehicleEvaluation>, RepositoryError>;
    async fn total_consumed_water(&self) -> Result<f64, RepositoryError>;
    async fn watering_plan_user(&self) -> Result<u64, RepositoryError>;
}
