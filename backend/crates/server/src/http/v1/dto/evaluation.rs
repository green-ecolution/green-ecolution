use serde::Serialize;

use domain::evaluation::{RegionEvaluation, VehicleEvaluation};

/// Evaluation metrics for a single region.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegionEvaluationResponse {
    /// Name of the region.
    #[schema(example = "Stadtpark Flensburg")]
    pub name: String,

    /// Number of watering plans associated with this region.
    #[schema(example = 5, minimum = 0)]
    pub watering_plan_count: i32,
}

impl From<&RegionEvaluation> for RegionEvaluationResponse {
    fn from(value: &RegionEvaluation) -> Self {
        Self {
            name: value.name.clone(),
            watering_plan_count: value.watering_plan_count,
        }
    }
}

/// Evaluation metrics for a single vehicle.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VehicleEvaluationResponse {
    /// License plate number of the vehicle.
    #[schema(example = "FL-GE 123")]
    pub number_plate: String,

    /// Number of watering plans this vehicle has been assigned to.
    #[schema(example = 12, minimum = 0)]
    pub watering_plan_count: i32,
}

impl From<&VehicleEvaluation> for VehicleEvaluationResponse {
    fn from(value: &VehicleEvaluation) -> Self {
        Self {
            number_plate: value.number_plate.clone(),
            watering_plan_count: value.watering_plan_count,
        }
    }
}

/// Aggregated evaluation metrics across the entire system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EvaluationResponse {
    /// Total number of trees in the system.
    #[schema(example = 342, minimum = 0)]
    pub tree_count: u32,

    /// Total number of tree clusters.
    #[schema(example = 28, minimum = 0)]
    pub treecluster_count: u32,

    /// Total number of sensors deployed.
    #[schema(example = 85, minimum = 0)]
    pub sensor_count: u32,

    /// Total number of watering plans.
    #[schema(example = 15, minimum = 0)]
    pub watering_plan_count: i32,

    /// Number of user-to-watering-plan assignments across all watering plans
    /// visible to the caller.
    #[schema(example = 8, minimum = 0)]
    pub user_watering_plan_count: u32,

    /// Total water consumption in liters.
    #[schema(example = 125000, minimum = 0)]
    pub total_water_consumption: u64,

    /// Per-region evaluation breakdown.
    pub region_evaluation: Vec<RegionEvaluationResponse>,

    /// Per-vehicle evaluation breakdown.
    pub vehicle_evaluation: Vec<VehicleEvaluationResponse>,
}
