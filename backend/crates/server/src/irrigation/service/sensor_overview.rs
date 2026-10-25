use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::sensor::{SensorId, data::SoilMoistureBucket};
use irrigation_domain::{
    cluster::{SoilMoistureOverview, TreeClusterReader, condition_series},
    tree::{TreeReader, volumetric_thresholds},
};

use crate::service::{ServiceError, sensor_service::SensorService};

/// A sensor's soil-moisture series read against the soil of the cluster its
/// tree stands in.
pub struct IrrigationSensorService {
    sensors: Arc<SensorService>,
    trees: Arc<dyn TreeReader>,
    clusters: Arc<dyn TreeClusterReader>,
}

impl IrrigationSensorService {
    pub fn new(
        sensors: Arc<SensorService>,
        trees: Arc<dyn TreeReader>,
        clusters: Arc<dyn TreeClusterReader>,
    ) -> Self {
        Self {
            sensors,
            trees,
            clusters,
        }
    }

    /// Thresholds and the condition series are empty when the sensor hangs on
    /// no tree, the tree on no cluster, or the cluster's soil is unknown.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn soil_moisture_overview(
        &self,
        id: &SensorId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<SoilMoistureOverview, ServiceError> {
        let series = self
            .sensors
            .soil_moisture_series(id, from, to, bucket)
            .await?;
        let cluster_id = self
            .trees
            .by_sensor_id(id)
            .await?
            .and_then(|tree| tree.cluster_id());
        let (soil, watering_events) = match cluster_id {
            Some(cluster_id) => {
                let view = self.clusters.view_by_id(cluster_id).await?;
                let events = self.clusters.watering_events(cluster_id).await?;
                (view.soil_condition, events)
            }
            None => (None, Vec::new()),
        };
        let thresholds = soil
            .map(|s| {
                series
                    .iter()
                    .filter_map(|d| volumetric_thresholds(s, d.depth_cm))
                    .collect()
            })
            .unwrap_or_default();
        let condition = soil
            .map(|s| condition_series(&series, s))
            .unwrap_or_default();
        Ok(SoilMoistureOverview {
            bucket,
            series,
            thresholds,
            condition,
            watering_events,
        })
    }
}
