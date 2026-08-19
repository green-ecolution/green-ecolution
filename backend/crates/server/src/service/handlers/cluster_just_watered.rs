use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::service::event_bus::{EventHandler, EventHandlerError};
use domain::{
    Id,
    cluster::{TreeCluster, TreeClusterReader, TreeClusterWriter},
    events::DomainEvent,
    tree::{TreeReader, TreeWriter},
};

/// Flags a finished watering plan's clusters and their trees as
/// [`domain::shared::watering_status::WateringStatus::JustWatered`].
///
/// Only `WateringPlanFinished` triggers this; starting, canceling or failing a
/// plan leaves the status alone. The status is released again by
/// `ClusterService::expire_just_watered` once the grace period passes.
pub struct ClusterJustWateredHandler {
    cluster_reader: Arc<dyn TreeClusterReader>,
    cluster_writer: Arc<dyn TreeClusterWriter>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
}

impl ClusterJustWateredHandler {
    pub fn new(
        cluster_reader: Arc<dyn TreeClusterReader>,
        cluster_writer: Arc<dyn TreeClusterWriter>,
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
    ) -> Self {
        Self {
            cluster_reader,
            cluster_writer,
            tree_reader,
            tree_writer,
        }
    }

    async fn mark_cluster(
        &self,
        cluster_id: Id<TreeCluster>,
        finished_at: DateTime<Utc>,
    ) -> Result<(), EventHandlerError> {
        let mut cluster = self.cluster_reader.by_id(cluster_id).await?;
        let mut trees = self.tree_reader.by_ids(&cluster.tree_ids).await?;

        for tree in trees.iter_mut() {
            tree.mark_just_watered(finished_at);
            self.tree_writer.save(tree).await?;
        }

        cluster.mark_just_watered(finished_at);
        self.cluster_writer.save(&cluster).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl EventHandler for ClusterJustWateredHandler {
    fn name(&self) -> &str {
        "cluster_just_watered"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let DomainEvent::WateringPlanFinished {
            cluster_ids,
            finished_at,
            ..
        } = event
        else {
            return Ok(vec![]);
        };

        for cluster_id in cluster_ids {
            if let Err(error) = self.mark_cluster(*cluster_id, *finished_at).await {
                tracing::warn!(%error, cluster.id = %cluster_id, "skipping just-watered update");
            }
        }
        Ok(vec![])
    }
}
