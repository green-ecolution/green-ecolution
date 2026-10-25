use std::sync::Arc;

use chrono::{DateTime, Utc};

use domain::{Id, events::DomainEvent};
use irrigation_domain::{
    cluster::{TreeCluster, TreeClusterReader, TreeClusterWriter},
    tree::{TreeReader, TreeWriter},
};
use server::service::event_bus::{EventHandler, EventHandlerError};

/// Flags the clusters a finished irrigation plan completed, and their trees, as
/// [`irrigation_domain::watering_status::WateringStatus::JustWatered`].
///
/// Only `OperationPlanFinished` triggers this; starting, canceling or failing a
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
        let DomainEvent::OperationPlanFinished {
            module,
            finished_at,
            outcomes,
            ..
        } = event
        else {
            return Ok(vec![]);
        };
        if module.as_str() != crate::KEY {
            return Ok(vec![]);
        }

        let cluster_ids: Vec<Id<TreeCluster>> = outcomes
            .iter()
            .filter(|o| o.completed)
            .map(|o| Id::new(o.stop.value()))
            .collect();
        self.cluster_writer
            .mark_last_watered(&cluster_ids, *finished_at)
            .await?;
        for cluster_id in &cluster_ids {
            if let Err(error) = self.mark_cluster(*cluster_id, *finished_at).await {
                tracing::warn!(%error, cluster.id = %cluster_id, "skipping just-watered update");
            }
        }
        Ok(vec![])
    }
}
