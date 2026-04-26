use serde::Serialize;

use crate::domain::evaluation::{RegionEvaluation, VehicleEvaluation};

#[derive(Debug, Serialize)]
pub struct RegionEvaluationResponse {
    pub name: String,
    pub watering_plan_count: u32,
}

impl From<&RegionEvaluation> for RegionEvaluationResponse {
    fn from(value: &RegionEvaluation) -> Self {
        Self {
            name: value.name.clone(),
            watering_plan_count: value.watering_plan_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VehicleEvaluationResponse {
    pub number_plate: String,
    pub watering_plan_count: u32,
}

impl From<&VehicleEvaluation> for VehicleEvaluationResponse {
    fn from(value: &VehicleEvaluation) -> Self {
        Self {
            number_plate: value.number_plate.clone(),
            watering_plan_count: value.watering_plan_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EvaluationResponse {
    pub tree_count: u32,
    pub treecluster_count: u32,
    pub sensor_count: u32,
    pub watering_plan_count: u32,
    pub user_watering_plan_count: u32,
    pub total_water_consumption: u64,
    pub region_evaluation: Vec<RegionEvaluationResponse>,
    pub vehicle_evaluation: Vec<VehicleEvaluationResponse>,
}
