use std::sync::Arc;

use crate::domain::{
    Id,
    events::DomainEvent,
    shared::pagination::{Page, Pagination},
    watering_plan::{
        WateringPlan, WateringPlanCreate, WateringPlanQuery, WateringPlanRepository,
        WateringPlanStatus, WateringPlanUpdate,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct WateringPlanService {
    watering_plan_repo: Arc<dyn WateringPlanRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl WateringPlanService {
    pub fn new(
        watering_plan_repo: Arc<dyn WateringPlanRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            watering_plan_repo,
            event_bus,
        }
    }

    pub async fn all(
        &self,
        query: WateringPlanQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlan>, ServiceError> {
        Ok(self.watering_plan_repo.all(query, pagination).await?)
    }

    pub async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        Ok(self.watering_plan_repo.by_id(id).await?)
    }

    pub async fn create(
        &self,
        input: WateringPlanCreate,
    ) -> Result<WateringPlan, ServiceError> {
        Ok(self.watering_plan_repo.create(input).await?)
    }

    pub async fn update(
        &self,
        id: Id<WateringPlan>,
        input: WateringPlanUpdate,
    ) -> Result<WateringPlan, ServiceError> {
        let new_status = input.status;
        let plan = self.watering_plan_repo.update(id, input).await?;

        if let Some(status @ WateringPlanStatus::Finished) = new_status {
            self.event_bus
                .publish(DomainEvent::WateringPlanStatusChanged {
                    plan_id: plan.id,
                    cluster_ids: plan.cluster_ids.clone(),
                    new_status: status,
                })
                .await;
        }

        Ok(plan)
    }

    pub async fn delete(&self, id: Id<WateringPlan>) -> Result<(), ServiceError> {
        Ok(self.watering_plan_repo.delete(id).await?)
    }
}
