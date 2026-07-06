use std::sync::Arc;

use chrono::Utc;

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    cluster::{SoilCondition, TreeClusterReader},
    events::DomainEvent,
    sensor::SensorReadingReader,
    tree::{TreeReader, TreeWriter},
};

/// Recomputes the volumetric watering status of every tree in a cluster
/// when the cluster's `soil_condition` changes.
///
/// Each tree with a sensor gets a fresh status from the latest stored
/// moisture reading under the new soil type. Trees without a sensor, or
/// without moisture data for the new soil, are skipped — the bus chains the
/// resulting `TreeWateringStatusChanged` events into the cluster-status
/// aggregator.
pub struct ClusterSoilRecalcHandler {
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    reading_reader: Arc<dyn SensorReadingReader>,
}

impl ClusterSoilRecalcHandler {
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
}

#[async_trait::async_trait]
impl EventHandler for ClusterSoilRecalcHandler {
    fn name(&self) -> &str {
        "cluster_soil_recalc"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let DomainEvent::ClusterSoilConditionChanged { cluster_id } = event else {
            return Ok(vec![]);
        };

        let cluster = match self.cluster_reader.by_id(*cluster_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping soil recalc; cluster load failed");
                return Ok(vec![]);
            }
        };

        let soil = cluster.soil_condition.unwrap_or(SoilCondition::Unknown);
        let trees = match self.tree_reader.by_ids(&cluster.tree_ids).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping soil recalc; tree load failed");
                return Ok(vec![]);
            }
        };

        let mut follow_ups = Vec::new();
        for mut tree in trees {
            let Some(sensor_id) = tree.sensor_id() else {
                continue;
            };
            let readings = match self
                .reading_reader
                .latest_volumetric_moisture(sensor_id)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(error = %e, sensor.id = %sensor_id, "skipping tree; reading fetch failed");
                    continue;
                }
            };
            let new_status = match tree.calculate_watering_status_from_volumetric(
                &readings,
                soil,
                Utc::now(),
            ) {
                Ok(s) => s,
                Err(e) => {
                    tracing::debug!(error = %e, sensor.id = %sensor_id, "skipping tree; calibration rejected");
                    continue;
                }
            };
            let evs = tree.record_watering_status(new_status);
            if !evs.is_empty() {
                self.tree_writer.save(&tree).await?;
                follow_ups.extend(evs);
            }
        }

        Ok(follow_ups)
    }
}
