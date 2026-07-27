use std::sync::Arc;

use domain::{
    authorization::Visibility,
    evaluation::{EvaluationRepository, RegionEvaluation, VehicleEvaluation},
};

use super::ServiceError;

pub struct EvaluationService {
    evaluation_repo: Arc<dyn EvaluationRepository>,
}

impl EvaluationService {
    pub fn new(evaluation_repo: Arc<dyn EvaluationRepository>) -> Self {
        Self { evaluation_repo }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn regions_with_watering_plan(
        &self,
        visible: Visibility,
    ) -> Result<Vec<RegionEvaluation>, ServiceError> {
        Ok(self
            .evaluation_repo
            .regions_with_watering_plan(visible)
            .await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn vehicle_with_watering_plan(
        &self,
        visible_vehicle: Visibility,
        visible_plan: Visibility,
    ) -> Result<Vec<VehicleEvaluation>, ServiceError> {
        Ok(self
            .evaluation_repo
            .vehicle_with_watering_plan(visible_vehicle, visible_plan)
            .await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn total_consumed_water(&self, visible: Visibility) -> Result<f64, ServiceError> {
        Ok(self.evaluation_repo.total_consumed_water(visible).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn watering_plan_user(&self, visible: Visibility) -> Result<u64, ServiceError> {
        Ok(self.evaluation_repo.watering_plan_user(visible).await?)
    }
}
