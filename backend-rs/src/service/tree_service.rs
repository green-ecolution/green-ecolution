use std::sync::Arc;

use crate::domain::{
    Id,
    cluster::TreeCluster,
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
    },
    tree::{
        PlantingYear, Tree, TreeCreate, TreeQuery, TreeRepository, TreeUpdate, TreeWithDistance,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct TreeService {
    tree_repo: Arc<dyn TreeRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl TreeService {
    pub fn new(tree_repo: Arc<dyn TreeRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            tree_repo,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn all(
        &self,
        query: TreeQuery,
        pagination: Pagination,
    ) -> Result<Page<Tree>, ServiceError> {
        Ok(self.tree_repo.all(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn by_id(&self, id: Id<Tree>) -> Result<Tree, ServiceError> {
        Ok(self.tree_repo.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, ServiceError> {
        Ok(self.tree_repo.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
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

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
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

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn archive(&self, id: Id<Tree>) -> Result<(), ServiceError> {
        Ok(self.tree_repo.archive(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(limit))]
    pub async fn nearest_trees(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeWithDistance>, ServiceError> {
        Ok(self.tree_repo.nearest_trees(coord, radius, limit).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, ServiceError> {
        Ok(self.tree_repo.distinct_planting_years().await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %cluster_id))]
    pub async fn unlink_cluster_id(&self, cluster_id: Id<TreeCluster>) -> Result<(), ServiceError> {
        Ok(self.tree_repo.unlink_cluster_id(cluster_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = sensor_id))]
    pub async fn unlink_sensor_id(&self, sensor_id: &str) -> Result<(), ServiceError> {
        Ok(self.tree_repo.unlink_sensor_id(sensor_id).await?)
    }
}
