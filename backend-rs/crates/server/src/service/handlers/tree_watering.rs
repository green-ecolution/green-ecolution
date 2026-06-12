use std::sync::Arc;

use chrono::Utc;

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    cluster::{SoilCondition, TreeClusterReader},
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    sensor::SensorReadingReader,
    tree::{TreeReader, TreeWriter},
};

/// Subscriber that turns each `SensorDataReceived` event into a fresh
/// watering-status decision on the linked tree.
///
/// Watermark readings keep using the event payload. Volumetric (soil-moisture)
/// readings are re-read from `sensor_data_ability_values` and scored against
/// the cluster's KA5 soil type and the tree's age.
pub struct TreeWateringFromSensorHandler {
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    reading_reader: Arc<dyn SensorReadingReader>,
}

impl TreeWateringFromSensorHandler {
    pub fn new(
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        cluster_reader: Arc<dyn TreeClusterReader>,
        reading_reader: Arc<dyn SensorReadingReader>,
    ) -> Self {
        Self {
            tree_reader,
            tree_writer,
            cluster_reader,
            reading_reader,
        }
    }

    async fn handle_inner(
        &self,
        payload: &SensorDataReceivedPayload,
    ) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let Some(mut tree) = self.tree_reader.by_sensor_id(&payload.sensor_id).await? else {
            return Ok(vec![]);
        };

        let outcome = match &payload.readings {
            SensorReadings::Watermarks(w) => {
                tree.calculate_watering_status_from_watermarks(w, Utc::now())
            }
            SensorReadings::Volumetrics(_) => {
                // Soil type flows from the linked cluster; without one we can't calibrate.
                let Some(cluster_id) = tree.cluster_id() else {
                    tracing::debug!("skipping volumetric status; tree has no cluster");
                    return Ok(vec![]);
                };
                let soil = self
                    .cluster_reader
                    .by_id(cluster_id)
                    .await?
                    .soil_condition
                    .unwrap_or(SoilCondition::Unknown);
                let readings = self
                    .reading_reader
                    .latest_volumetric_moisture(&payload.sensor_id)
                    .await?;
                tree.calculate_watering_status_from_volumetric(&readings, soil, Utc::now())
            }
        };

        let new_status = match outcome {
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
        let DomainEvent::SensorDataReceived(payload) = event else {
            return Ok(vec![]);
        };
        self.handle_inner(payload).await
    }
}
