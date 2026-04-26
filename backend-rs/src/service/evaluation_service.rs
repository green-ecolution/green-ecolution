use std::sync::Arc;

use crate::domain::evaluation::{
    EvaluationRepository, RegionEvaluation, VehicleEvaluation,
};

use super::ServiceError;

pub struct EvaluationService {
    evaluation_repo: Arc<dyn EvaluationRepository>,
}

impl EvaluationService {
    pub fn new(evaluation_repo: Arc<dyn EvaluationRepository>) -> Self {
        Self { evaluation_repo }
    }

    pub async fn regions_with_watering_plan(
        &self,
    ) -> Result<Vec<RegionEvaluation>, ServiceError> {
        Ok(self.evaluation_repo.regions_with_watering_plan().await?)
    }

    pub async fn vehicle_with_watering_plan(
        &self,
    ) -> Result<Vec<VehicleEvaluation>, ServiceError> {
        Ok(self.evaluation_repo.vehicle_with_watering_plan().await?)
    }

    pub async fn total_consumed_water(&self) -> Result<f64, ServiceError> {
        Ok(self.evaluation_repo.total_consumed_water().await?)
    }

    pub async fn watering_plan_user(&self) -> Result<u64, ServiceError> {
        Ok(self.evaluation_repo.watering_plan_user().await?)
    }
}
