use std::sync::Arc;

use domain::{
    Id,
    operation_plan::{OperationPlan, OperationPlanReader, OperationPlanWriter},
    planning::StopOutcome,
};

use super::{ServiceError, event_bus::EventBus};

pub struct OperationExecutionService {
    reader: Arc<dyn OperationPlanReader>,
    writer: Arc<dyn OperationPlanWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl OperationExecutionService {
    pub fn new(
        reader: Arc<dyn OperationPlanReader>,
        writer: Arc<dyn OperationPlanWriter>,
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
        id: Id<OperationPlan>,
        outcomes: Vec<StopOutcome>,
    ) -> Result<OperationPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let events = plan.finish(&outcomes)?;
        self.writer.save_finished(&plan, &outcomes).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }
}
