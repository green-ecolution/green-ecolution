use std::sync::Arc;

use chrono::Utc;

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    events::DomainEvent,
    sensor::{SensorId, data::Watermark},
    tree::{TreeReader, TreeWriter},
};

/// Subscriber that turns each `SensorDataReceived` event into a fresh
/// watering-status decision on the linked tree.
///
/// Looks up the tree currently bound to the reading's sensor, asks the tree
/// aggregate to score the watermarks against its calibration table, and
/// returns the `TreeWateringStatusChanged` events the aggregate produces (the
/// bus chains them so the cluster status aggregator picks them up).
pub struct TreeWateringFromSensorHandler {
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
}

impl TreeWateringFromSensorHandler {
    pub fn new(tree_reader: Arc<dyn TreeReader>, tree_writer: Arc<dyn TreeWriter>) -> Self {
        Self {
            tree_reader,
            tree_writer,
        }
    }

    async fn handle_inner(
        &self,
        sensor_id: &SensorId,
        watermarks: &[Watermark],
    ) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let Some(mut tree) = self.tree_reader.by_sensor_id(sensor_id).await? else {
            return Ok(vec![]);
        };
        let new_status = match tree.calculate_watering_status(watermarks, Utc::now()) {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!(error = %e, "skipping tree watering update; calibration rejected payload");
                return Ok(vec![]);
            }
        };
        let events = tree.record_watering_status(new_status);
        if events.is_empty() {
            return Ok(vec![]);
        }
        self.tree_writer.save(&tree).await?;
        Ok(events)
    }
}

#[async_trait::async_trait]
impl EventHandler for TreeWateringFromSensorHandler {
    fn name(&self) -> &str {
        "tree_watering_from_sensor"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let DomainEvent::SensorDataReceived {
            sensor_id,
            watermarks,
            ..
        } = event
        else {
            return Ok(vec![]);
        };
        self.handle_inner(sensor_id, watermarks).await
    }
}
