use std::sync::Arc;

use crate::domain::{
    Id,
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    tree::{
        PlantingYear, Tree, TreeCreate, TreeQuery, TreeRepository, TreeUpdate, TreeWithDistance,
    },
    cluster::TreeCluster,
};

use super::{ServiceError, event_bus::EventBus};

pub struct TreeService {
    tree_repo: Arc<dyn TreeRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl TreeService {
    pub fn new(tree_repo: Arc<dyn TreeRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self { tree_repo, event_bus }
    }

    pub async fn all(
        &self,
        query: TreeQuery,
        pagination: Pagination,
    ) -> Result<Page<Tree>, ServiceError> {
        Ok(self.tree_repo.all(query, pagination).await?)
    }

    pub async fn by_id(&self, id: Id<Tree>) -> Result<Tree, ServiceError> {
        Ok(self.tree_repo.by_id(id).await?)
    }

    pub async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, ServiceError> {
        Ok(self.tree_repo.by_ids(ids).await?)
    }

    pub async fn create(&self, input: TreeCreate) -> Result<Tree, ServiceError> {
        let tree = self.tree_repo.create(input).await?;
        self.event_bus
            .publish(DomainEvent::TreeCreated {
                tree_id: tree.id,
                cluster_id: tree.cluster_id,
            })
            .await;
        Ok(tree)
    }

    pub async fn update(&self, id: Id<Tree>, input: TreeUpdate) -> Result<Tree, ServiceError> {
        let old = self.tree_repo.by_id(id).await?;
        let tree = self.tree_repo.update(id, input).await?;
        self.event_bus
            .publish(DomainEvent::TreeUpdated {
                tree_id: tree.id,
                old_cluster_id: old.cluster_id,
                new_cluster_id: tree.cluster_id,
            })
            .await;
        Ok(tree)
    }

    pub async fn delete(&self, id: Id<Tree>) -> Result<(), ServiceError> {
        let tree = self.tree_repo.by_id(id).await?;
        self.tree_repo.delete(id).await?;
        self.event_bus
            .publish(DomainEvent::TreeDeleted {
                tree_id: id,
                cluster_id: tree.cluster_id,
            })
            .await;
        Ok(())
    }

    pub async fn archive(&self, id: Id<Tree>) -> Result<(), ServiceError> {
        Ok(self.tree_repo.archive(id).await?)
    }

    pub async fn nearest_trees(
        &self,
        coord: Coordinate,
        radius_meters: f64,
        limit: u32,
    ) -> Result<Vec<TreeWithDistance>, ServiceError> {
        Ok(self.tree_repo.nearest_trees(coord, radius_meters, limit).await?)
    }

    pub async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, ServiceError> {
        Ok(self.tree_repo.distinct_planting_years().await?)
    }

    pub async fn unlink_cluster_id(
        &self,
        cluster_id: Id<TreeCluster>,
    ) -> Result<(), ServiceError> {
        Ok(self.tree_repo.unlink_cluster_id(cluster_id).await?)
    }

    pub async fn unlink_sensor_id(&self, sensor_id: &str) -> Result<(), ServiceError> {
        Ok(self.tree_repo.unlink_sensor_id(sensor_id).await?)
    }
}
