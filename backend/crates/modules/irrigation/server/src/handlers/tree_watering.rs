use std::sync::Arc;

use chrono::Utc;

use crate::targets::is_tree;
use domain::{
    Id, RepositoryError,
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    sensor::SensorReadingReader,
};
use irrigation_domain::{
    cluster::{SoilCondition, TreeClusterReader},
    tree::{TreeError, TreeReader, TreeWriter},
    watering_status::WateringStatus,
};
use server::service::event_bus::{EventHandler, EventHandlerError};

/// Turns each `SensorDataReceived` event into a fresh watering-status decision
/// on the tree the sensor is attached to.
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
        let Some(target) = payload.target.as_ref().filter(|t| is_tree(t)) else {
            return Ok(vec![]);
        };
        let mut tree = match self.tree_reader.by_id(Id::new(target.id)).await {
            Ok(tree) => tree,
            Err(RepositoryError::NotFound) => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };

        let outcome = match &payload.readings {
            SensorReadings::Watermarks(w) => {
                tree.calculate_watering_status_from_watermarks(w, Utc::now())
            }
            SensorReadings::Distances(_) => return Ok(vec![]),
            SensorReadings::Volumetrics(_) => {
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
            // No calibration applies, so a stored status can never be refreshed.
            // A tree still in the future has no soil to measure at all.
            Err(
                TreeError::UncalibratedSoil
                | TreeError::BeyondMonitoring
                | TreeError::NotYetPlanted,
            ) => WateringStatus::Unknown,
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
