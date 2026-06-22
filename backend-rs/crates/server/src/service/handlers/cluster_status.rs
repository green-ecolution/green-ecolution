use std::sync::Arc;

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    Id,
    cluster::{TreeCluster, TreeClusterReader, TreeClusterWriter},
    events::DomainEvent,
    tree::TreeReader,
};

pub struct ClusterStatusAggregatorHandler {
    cluster_reader: Arc<dyn TreeClusterReader>,
    cluster_writer: Arc<dyn TreeClusterWriter>,
    tree_reader: Arc<dyn TreeReader>,
}

impl ClusterStatusAggregatorHandler {
    pub fn new(
        cluster_reader: Arc<dyn TreeClusterReader>,
        cluster_writer: Arc<dyn TreeClusterWriter>,
        tree_reader: Arc<dyn TreeReader>,
    ) -> Self {
        Self {
            cluster_reader,
            cluster_writer,
            tree_reader,
        }
    }

    fn affected_cluster_ids(&self, event: &DomainEvent) -> Vec<Id<TreeCluster>> {
        match event {
            DomainEvent::TreeCreated {
                cluster_id,
                sensor_id: Some(_),
                ..
            } => cluster_id.iter().copied().collect(),
            DomainEvent::TreeDeleted {
                cluster_id,
                had_sensor: true,
                ..
            } => cluster_id.iter().copied().collect(),
            DomainEvent::TreeMovedBetweenClusters { from, to, .. } => {
                let mut ids: Vec<_> = [*from, *to].into_iter().flatten().collect();
                ids.dedup();
                ids
            }
            DomainEvent::TreeSensorAttached { cluster_id, .. }
            | DomainEvent::TreeSensorDetached { cluster_id, .. }
            | DomainEvent::TreeWateringStatusChanged { cluster_id, .. } => {
                cluster_id.iter().copied().collect()
            }
            DomainEvent::ClusterTreesChanged { cluster_id } => vec![*cluster_id],
            _ => vec![],
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for ClusterStatusAggregatorHandler {
    fn name(&self) -> &str {
        "cluster_status_aggregator"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        for cluster_id in self.affected_cluster_ids(event) {
            let mut cluster = match self.cluster_reader.by_id(cluster_id).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping cluster status update; load failed");
                    continue;
                }
            };

            let trees = match self.tree_reader.by_ids(&cluster.tree_ids).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping cluster status update; tree load failed");
                    continue;
                }
            };
            let statuses: Vec<_> = trees
                .iter()
                .filter(|t| t.sensor_id().is_some())
                .map(|t| t.watering_status())
                .collect();

            cluster.recalculate_watering_status(&statuses);
            self.cluster_writer.save(&cluster).await?;
        }
        Ok(vec![])
    }
}
