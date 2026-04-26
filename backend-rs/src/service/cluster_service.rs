use std::sync::Arc;

use crate::domain::{
    Id,
    cluster::{
        TreeCluster, TreeClusterCreate, TreeClusterQuery, TreeClusterRepository,
        TreeClusterUpdate,
    },
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    tree::TreeRepository,
};

use super::{ServiceError, event_bus::EventBus};

pub struct ClusterService {
    cluster_repo: Arc<dyn TreeClusterRepository>,
    tree_repo: Arc<dyn TreeRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl ClusterService {
    pub fn new(
        cluster_repo: Arc<dyn TreeClusterRepository>,
        tree_repo: Arc<dyn TreeRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            cluster_repo,
            tree_repo,
            event_bus,
        }
    }

    pub async fn all(
        &self,
        query: TreeClusterQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeCluster>, ServiceError> {
        Ok(self.cluster_repo.all(query, pagination).await?)
    }

    pub async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, ServiceError> {
        Ok(self.cluster_repo.by_id(id).await?)
    }

    pub async fn by_ids(
        &self,
        ids: &[Id<TreeCluster>],
    ) -> Result<Vec<TreeCluster>, ServiceError> {
        Ok(self.cluster_repo.by_ids(ids).await?)
    }

    pub async fn create(
        &self,
        input: TreeClusterCreate,
    ) -> Result<TreeCluster, ServiceError> {
        let cluster = self.cluster_repo.create(input).await?;
        self.event_bus
            .publish(DomainEvent::ClusterTreesChanged {
                cluster_id: cluster.id,
            })
            .await;
        Ok(cluster)
    }

    pub async fn update(
        &self,
        id: Id<TreeCluster>,
        input: TreeClusterUpdate,
    ) -> Result<(), ServiceError> {
        let trees_changed = input.tree_ids.is_some();
        self.cluster_repo.update(id, input).await?;
        if trees_changed {
            self.event_bus
                .publish(DomainEvent::ClusterTreesChanged { cluster_id: id })
                .await;
        }
        Ok(())
    }

    pub async fn delete(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        self.tree_repo.unlink_cluster_id(id).await?;
        self.cluster_repo.delete(id).await?;
        Ok(())
    }

    pub async fn archive(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        Ok(self.cluster_repo.archive(id).await?)
    }

    pub async fn center_point(&self, id: Id<TreeCluster>) -> Result<Coordinate, ServiceError> {
        Ok(self.cluster_repo.center_point(id).await?)
    }
}
