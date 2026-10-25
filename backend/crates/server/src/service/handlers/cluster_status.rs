use std::sync::Arc;

use crate::{
    irrigation::targets::is_tree,
    service::event_bus::{EventHandler, EventHandlerError},
};
use domain::{
    Id, RepositoryError,
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

    async fn affected_cluster_ids(
        &self,
        event: &DomainEvent,
    ) -> Result<Vec<Id<TreeCluster>>, EventHandlerError> {
        Ok(match event {
            DomainEvent::TreeCreated { cluster_id, .. }
            | DomainEvent::TreeDeleted { cluster_id, .. }
            | DomainEvent::TreeWateringStatusChanged { cluster_id, .. } => {
                cluster_id.iter().copied().collect()
            }
            DomainEvent::TreeMovedBetweenClusters { from, to, .. } => {
                let mut ids: Vec<_> = [*from, *to].into_iter().flatten().collect();
                ids.dedup();
                ids
            }
            DomainEvent::SensorAttached { target, .. }
            | DomainEvent::SensorDetached { target, .. }
                if is_tree(target) =>
            {
                match self.tree_reader.by_id(Id::new(target.id)).await {
                    Ok(tree) => tree.cluster_id().into_iter().collect(),
                    Err(RepositoryError::NotFound) => vec![],
                    Err(e) => return Err(e.into()),
                }
            }
            DomainEvent::ClusterTreesChanged { cluster_id } => vec![*cluster_id],
            _ => vec![],
        })
    }
}

#[async_trait::async_trait]
impl EventHandler for ClusterStatusAggregatorHandler {
    fn name(&self) -> &str {
        "cluster_status_aggregator"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        for cluster_id in self.affected_cluster_ids(event).await? {
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
            let with_sensor = match self.tree_reader.sensor_ids(&cluster.tree_ids).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping cluster status update; sensor load failed");
                    continue;
                }
            };
            let statuses: Vec<_> = trees
                .iter()
                .filter(|t| with_sensor.contains_key(&t.id))
                .map(|t| t.watering_status())
                .collect();

            cluster.recalculate_watering_status(&statuses);
            self.cluster_writer.save(&cluster).await?;
        }
        Ok(vec![])
    }
}
