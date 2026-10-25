use std::sync::Arc;

use domain::{Id, events::DomainEvent, region::RegionReader};
use irrigation_domain::{
    cluster::{TreeCluster, TreeClusterReader, TreeClusterWriter},
    events::IrrigationEvent,
    tree::TreeReader,
};
use server::service::event_bus::{EventHandler, EventHandlerError};

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
        match IrrigationEvent::from_domain(event) {
            Some(IrrigationEvent::TreeCreated { cluster_id, .. }) => {
                cluster_id.iter().copied().collect()
            }
            Some(IrrigationEvent::TreeDeleted { cluster_id, .. }) => {
                cluster_id.iter().copied().collect()
            }
            Some(IrrigationEvent::TreeCoordinateChanged { cluster_id, .. }) => {
                cluster_id.iter().copied().collect()
            }
            Some(IrrigationEvent::TreeMovedBetweenClusters { from, to, .. }) => {
                let mut ids: Vec<_> = [*from, *to].into_iter().flatten().collect();
                ids.dedup();
                ids
            }
            Some(IrrigationEvent::ClusterTreesChanged { cluster_id }) => vec![*cluster_id],
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
                    tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping cluster recalc; load failed");
                    continue;
                }
            };

            let trees = match self.tree_reader.by_ids(&cluster.tree_ids).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(error = %e, cluster.id = %cluster_id, "skipping cluster recalc; tree load failed");
                    continue;
                }
            };
            let coords: Vec<_> = trees.iter().map(|t| t.coordinate).collect();
            cluster.recalculate_centroid(&coords);

            let region_id = match cluster.coordinates() {
                Some(center) => match self.region_reader.by_point(center).await {
                    Ok(r) => r.map(|r| r.id),
                    Err(e) => {
                        tracing::warn!(error = %e, cluster.id = %cluster_id, "region lookup failed; clearing region");
                        None
                    }
                },
                None => None,
            };
            cluster.assign_region(region_id);

            self.cluster_writer.save(&cluster).await?;
            self.cluster_writer.refresh_boundary(cluster_id).await?;
        }
        Ok(vec![])
    }
}
