use std::sync::Arc;

use domain::{
    Id,
    events::DomainEvent,
    shared::pagination::{Page, Pagination},
    watering_plan::{
        WateringPlan, WateringPlanDraft, WateringPlanError, WateringPlanEvaluation,
        WateringPlanReader, WateringPlanSearchQuery, WateringPlanUpdate, WateringPlanView,
        WateringPlanWriter,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct WateringPlanService {
    reader: Arc<dyn WateringPlanReader>,
    writer: Arc<dyn WateringPlanWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl WateringPlanService {
    pub fn new(
        reader: Arc<dyn WateringPlanReader>,
        writer: Arc<dyn WateringPlanWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: WateringPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlanView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn view_by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlanView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn evaluations(
        &self,
        id: Id<WateringPlan>,
    ) -> Result<Vec<WateringPlanEvaluation>, ServiceError> {
        Ok(self.reader.evaluations(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: WateringPlanDraft) -> Result<WateringPlan, ServiceError> {
        Ok(self.writer.save_new(draft).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn replace_details(
        &self,
        id: Id<WateringPlan>,
        update: WateringPlanUpdate,
    ) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        plan.replace_details(update).map_err(map_plan_error)?;
        self.writer.save(&plan).await?;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn start(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let events = plan.start().map_err(map_plan_error)?;
        self.writer.save(&plan).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn cancel(
        &self,
        id: Id<WateringPlan>,
        note: String,
    ) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let events = plan.cancel(note).map_err(map_plan_error)?;
        self.writer.save(&plan).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn fail(
        &self,
        id: Id<WateringPlan>,
        note: String,
    ) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let events = plan.fail(note).map_err(map_plan_error)?;
        self.writer.save(&plan).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn delete(&self, id: Id<WateringPlan>) -> Result<(), ServiceError> {
        let plan = self.reader.by_id(id).await?;
        let cluster_ids = plan.cluster_ids().to_vec();
        self.writer.delete(id).await?;
        self.event_bus
            .publish(DomainEvent::WateringPlanDeleted {
                plan_id: id,
                cluster_ids,
            })
            .await;
        Ok(())
    }
}

fn map_plan_error(e: WateringPlanError) -> ServiceError {
    ServiceError::InvalidInput(e.to_string())
}
