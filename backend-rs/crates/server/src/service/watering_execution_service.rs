use std::sync::Arc;

use chrono::Utc;

use domain::{
    Id,
    watering_plan::{WateringPlan, WateringPlanEvaluation, WateringPlanReader, WateringPlanWriter},
};

use super::{ServiceError, event_bus::EventBus};

pub struct WateringExecutionService {
    reader: Arc<dyn WateringPlanReader>,
    writer: Arc<dyn WateringPlanWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl WateringExecutionService {
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

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn finish(
        &self,
        id: Id<WateringPlan>,
        evaluations: Vec<WateringPlanEvaluation>,
    ) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let events = plan
            .finish(&evaluations)
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;

        let cluster_ids: Vec<_> = plan.cluster_ids().to_vec();
        let now = Utc::now();

        self.writer.save(&plan).await?;
        self.writer.save_evaluations(plan.id, &evaluations).await?;
        self.writer
            .propagate_last_watered(&cluster_ids, now)
            .await?;

        self.event_bus.publish_all(events).await;

        Ok(plan)
    }
}
