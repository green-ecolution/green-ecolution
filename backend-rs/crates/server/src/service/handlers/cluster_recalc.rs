use std::sync::Arc;

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    Id,
    cluster::{TreeCluster, TreeClusterReader, TreeClusterWriter},
    events::DomainEvent,
    region::RegionReader,
    tree::TreeReader,
};

pub struct ClusterRecalculationHandler {
    cluster_reader: Arc<dyn TreeClusterReader>,
    cluster_writer: Arc<dyn TreeClusterWriter>,
    tree_reader: Arc<dyn TreeReader>,
    region_reader: Arc<dyn RegionReader>,
}

impl ClusterRecalculationHandler {
    pub fn new(
        cluster_reader: Arc<dyn TreeClusterReader>,
        cluster_writer: Arc<dyn TreeClusterWriter>,
        tree_reader: Arc<dyn TreeReader>,
        region_reader: Arc<dyn RegionReader>,
    ) -> Self {
        Self {
            cluster_reader,
            cluster_writer,
            tree_reader,
            region_reader,
        }
    }

    fn affected_cluster_ids(&self, event: &DomainEvent) -> Vec<Id<TreeCluster>> {
        match event {
            DomainEvent::TreeCreated { cluster_id, .. } => cluster_id.iter().copied().collect(),
            DomainEvent::TreeDeleted { cluster_id, .. } => cluster_id.iter().copied().collect(),
            DomainEvent::TreeCoordinateChanged { cluster_id, .. } => {
                cluster_id.iter().copied().collect()
            }
            DomainEvent::TreeMovedBetweenClusters { from, to, .. } => {
                let mut ids: Vec<_> = [*from, *to].into_iter().flatten().collect();
                ids.dedup();
                ids
            }
            DomainEvent::ClusterTreesChanged { cluster_id } => vec![*cluster_id],
            _ => vec![],
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for ClusterRecalculationHandler {
    fn name(&self) -> &str {
        "cluster_recalculation"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        for cluster_id in self.affected_cluster_ids(event) {
            let mut cluster = match self.cluster_reader.by_id(cluster_id).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(error = %e, %cluster_id, "skipping cluster recalc; load failed");
                    continue;
                }
            };

            let trees = match self.tree_reader.by_ids(&cluster.tree_ids).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(error = %e, %cluster_id, "skipping cluster recalc; tree load failed");
                    continue;
                }
            };
            let coords: Vec<_> = trees.iter().map(|t| t.coordinate).collect();
            cluster.recalculate_centroid(&coords);

            let region_id = match cluster.coordinates() {
                Some(center) => match self.region_reader.by_point(center).await {
                    Ok(r) => r.map(|r| r.id),
                    Err(e) => {
                        tracing::warn!(error = %e, %cluster_id, "region lookup failed; clearing region");
                        None
                    }
                },
                None => None,
            };
            cluster.assign_region(region_id);

            self.cluster_writer.save(&cluster).await?;
        }
        Ok(vec![])
    }
}
