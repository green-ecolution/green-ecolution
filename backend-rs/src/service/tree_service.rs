use std::sync::Arc;

use crate::domain::{
    Id,
    cluster::TreeCluster,
    events::DomainEvent,
    sensor::SensorId,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        watering_status::WateringStatus,
    },
    tree::{
        PlantingYear, Tree, TreeDraft, TreeReader, TreeSearchQuery, TreeView, TreeViewWithDistance,
        TreeWriter,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct TreeService {
    reader: Arc<dyn TreeReader>,
    writer: Arc<dyn TreeWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl TreeService {
    pub fn new(
        reader: Arc<dyn TreeReader>,
        writer: Arc<dyn TreeWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: TreeSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn by_id(&self, id: Id<Tree>) -> Result<Tree, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, ServiceError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn view_by_id(&self, id: Id<Tree>) -> Result<TreeView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<TreeView>, ServiceError> {
        Ok(self.reader.view_by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %sensor_id))]
    pub async fn view_by_sensor_id(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Option<TreeView>, ServiceError> {
        Ok(self.reader.view_by_sensor_id(sensor_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: TreeDraft) -> Result<Tree, ServiceError> {
        let tree = self.writer.save_new(draft).await?;
        self.event_bus
            .publish(DomainEvent::TreeCreated {
                tree_id: tree.id,
                cluster_id: tree.cluster_id(),
                sensor_id: tree.sensor_id().cloned(),
            })
            .await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn replace(&self, id: Id<Tree>, draft: TreeDraft) -> Result<Tree, ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        let mut events = Vec::new();
        events.extend(tree.replace_details(
            draft.species,
            draft.tree_number,
            draft.planting_year,
            draft.coordinate,
            draft.description,
            draft.provenance,
        ));
        events.extend(tree.move_to_cluster(draft.cluster_id));
        events.extend(match draft.sensor_id {
            Some(sid) => tree.attach_sensor(sid),
            None => tree.detach_sensor(),
        });
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn move_to_cluster(
        &self,
        id: Id<Tree>,
        target: Option<Id<TreeCluster>>,
    ) -> Result<Tree, ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        let events = tree.move_to_cluster(target);
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn attach_sensor(
        &self,
        id: Id<Tree>,
        sensor_id: SensorId,
    ) -> Result<Tree, ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        let events = tree.attach_sensor(sensor_id);
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn detach_sensor(&self, id: Id<Tree>) -> Result<Tree, ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        let events = tree.detach_sensor();
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn record_watering_status(
        &self,
        id: Id<Tree>,
        status: WateringStatus,
    ) -> Result<Tree, ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        let events = tree.record_watering_status(status);
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn delete(&self, id: Id<Tree>) -> Result<(), ServiceError> {
        let tree = self.reader.by_id(id).await?;
        let cluster_id = tree.cluster_id();
        let had_sensor = tree.had_sensor();
        self.writer.delete(id).await?;
        self.event_bus
            .publish(DomainEvent::TreeDeleted {
                tree_id: id,
                cluster_id,
                had_sensor,
            })
            .await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(limit))]
    pub async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeViewWithDistance>, ServiceError> {
        Ok(self.reader.view_nearest(coord, radius, limit).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, ServiceError> {
        Ok(self.reader.distinct_planting_years().await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %cluster_id))]
    pub async fn unlink_cluster_id(&self, cluster_id: Id<TreeCluster>) -> Result<(), ServiceError> {
        Ok(self.writer.unlink_cluster_id(cluster_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %sensor_id))]
    pub async fn unlink_sensor_id(&self, sensor_id: &SensorId) -> Result<(), ServiceError> {
        Ok(self.writer.unlink_sensor_id(sensor_id).await?)
    }
}
