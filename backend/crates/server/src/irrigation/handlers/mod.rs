pub mod cluster_just_watered;
pub mod cluster_recalc;
pub mod cluster_soil_recalc;
pub mod cluster_status;
pub mod sensor_lifecycle;
pub mod tree_watering;

use std::sync::Arc;

use domain::{region::RegionReader, sensor::SensorReadingReader};
use irrigation_domain::{
    cluster::{TreeClusterReader, TreeClusterWriter},
    tree::{TreeReader, TreeWriter},
};

use crate::service::{event_bus::EventHandler, sensor_service::SensorService};

pub fn all(
    trees: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    clusters: Arc<dyn TreeClusterReader>,
    cluster_writer: Arc<dyn TreeClusterWriter>,
    regions: Arc<dyn RegionReader>,
    readings: Arc<dyn SensorReadingReader>,
    sensors: &Arc<SensorService>,
) -> Vec<Arc<dyn EventHandler>> {
    vec![
        Arc::new(cluster_recalc::ClusterRecalculationHandler::new(
            clusters.clone(),
            cluster_writer.clone(),
            trees.clone(),
            regions,
        )),
        Arc::new(cluster_soil_recalc::ClusterSoilRecalcHandler::new(
            trees.clone(),
            tree_writer.clone(),
            clusters.clone(),
            readings.clone(),
        )),
        Arc::new(cluster_just_watered::ClusterJustWateredHandler::new(
            clusters.clone(),
            cluster_writer.clone(),
            trees.clone(),
            tree_writer.clone(),
        )),
        Arc::new(cluster_status::ClusterStatusAggregatorHandler::new(
            clusters.clone(),
            cluster_writer,
            trees.clone(),
        )),
        Arc::new(tree_watering::TreeWateringFromSensorHandler::new(
            trees.clone(),
            tree_writer.clone(),
            clusters,
            readings,
        )),
        Arc::new(sensor_lifecycle::SensorLifecycle::new(
            trees,
            tree_writer,
            sensors,
        )),
    ]
}
