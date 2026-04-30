use std::sync::Arc;

use crate::domain::{
    cluster::{TreeClusterRepository, TreeClusterUpdate},
    events::DomainEvent,
    region::RegionRepository,
    shared::field_update::FieldUpdate,
};
use crate::service::event_bus::{EventHandler, EventHandlerError};

pub struct ClusterRecalculationHandler {
    cluster_repo: Arc<dyn TreeClusterRepository>,
    region_repo: Arc<dyn RegionRepository>,
}

impl ClusterRecalculationHandler {
    pub fn new(
        cluster_repo: Arc<dyn TreeClusterRepository>,
        region_repo: Arc<dyn RegionRepository>,
    ) -> Self {
        Self {
            cluster_repo,
            region_repo,
        }
    }

    fn affected_cluster_ids(
        &self,
        event: &DomainEvent,
    ) -> Vec<crate::domain::Id<crate::domain::cluster::TreeCluster>> {
        match event {
            DomainEvent::TreeCreated { cluster_id, .. } => cluster_id.iter().copied().collect(),
            DomainEvent::TreeUpdated {
                old_cluster_id,
                new_cluster_id,
                ..
            } => {
                let mut ids: Vec<_> = [*old_cluster_id, *new_cluster_id]
                    .into_iter()
                    .flatten()
                    .collect();
                ids.dedup();
                ids
            }
            DomainEvent::TreeDeleted { cluster_id, .. } => cluster_id.iter().copied().collect(),
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

    fn handles(&self, event: &DomainEvent) -> bool {
        !self.affected_cluster_ids(event).is_empty()
    }

    async fn handle(&self, event: &DomainEvent) -> Result<(), EventHandlerError> {
        for cluster_id in self.affected_cluster_ids(event) {
            let coordinates = self.cluster_repo.center_point(cluster_id).await.ok();
            let region_id = match coordinates {
                Some(center) => self.region_repo.by_point(center).await.ok().map(|r| r.id),
                None => None,
            };

            self.cluster_repo
                .update(
                    cluster_id,
                    TreeClusterUpdate {
                        coordinates: coordinates.map_or(FieldUpdate::Cleared, FieldUpdate::Set),
                        region_id: region_id.map_or(FieldUpdate::Cleared, FieldUpdate::Set),
                        ..Default::default()
                    },
                )
                .await?;
        }
        Ok(())
    }
}
