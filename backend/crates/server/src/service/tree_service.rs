use std::sync::Arc;

use domain::{
    Id,
    authorization::Visibility,
    cluster::{TreeCluster, TreeClusterReader},
    events::DomainEvent,
    organization::Organization,
    sensor::{SensorId, SensorReader, SensorWriter},
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        watering_status::WateringStatus,
    },
    tree::{
        PlantingYear, Tree, TreeDraft, TreeMarker, TreeReader, TreeSearchQuery, TreeView,
        TreeViewWithDistance, TreeWriter,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct TreeService {
    reader: Arc<dyn TreeReader>,
    writer: Arc<dyn TreeWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    sensor_reader: Arc<dyn SensorReader>,
    sensor_writer: Arc<dyn SensorWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl TreeService {
    pub fn new(
        reader: Arc<dyn TreeReader>,
        writer: Arc<dyn TreeWriter>,
        cluster_reader: Arc<dyn TreeClusterReader>,
        sensor_reader: Arc<dyn SensorReader>,
        sensor_writer: Arc<dyn SensorWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            cluster_reader,
            sensor_reader,
            sensor_writer,
            event_bus,
        }
    }

    /// Rejects moving a tree into a cluster owned by a different
    /// organization. `cluster_id = None` (leaving/staying out of a cluster)
    /// is always allowed.
    async fn ensure_cluster_matches_org(
        &self,
        cluster_id: Option<Id<TreeCluster>>,
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        if let Some(cid) = cluster_id {
            let cluster = self.cluster_reader.by_id(cid).await?;
            if cluster.organization_id() != org {
                return Err(ServiceError::OrganizationMismatch);
            }
        }
        Ok(())
    }

    /// Rejects attaching a sensor owned by a different organization than the
    /// tree it would be bound to.
    async fn ensure_sensor_matches_org(
        &self,
        sensor_id: &SensorId,
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let sensor = self.sensor_reader.by_id(sensor_id).await?;
        if sensor.organization_id() != org {
            return Err(ServiceError::OrganizationMismatch);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: TreeSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_markers(
        &self,
        query: TreeSearchQuery,
    ) -> Result<Vec<TreeMarker>, ServiceError> {
        Ok(self.reader.view_markers(query).await?)
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

    /// Rejects a sensor that is already linked to a different tree. Moving a
    /// sensor between trees is an explicit flow (`SensorService::reassign_tree`),
    /// not a side effect of tree edits.
    async fn ensure_sensor_unassigned(
        &self,
        sensor_id: &SensorId,
        tree_id: Option<Id<Tree>>,
    ) -> Result<(), ServiceError> {
        if let Some(holder) = self.reader.by_sensor_id(sensor_id).await?
            && Some(holder.id) != tree_id
        {
            return Err(ServiceError::SensorAlreadyAssigned);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: TreeDraft) -> Result<Tree, ServiceError> {
        if let Some(ref sid) = draft.sensor_id {
            self.ensure_sensor_unassigned(sid, None).await?;
            self.ensure_sensor_matches_org(sid, draft.organization_id)
                .await?;
        }
        self.ensure_cluster_matches_org(draft.cluster_id, draft.organization_id)
            .await?;
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
        if let Some(ref sid) = draft.sensor_id {
            self.ensure_sensor_unassigned(sid, Some(id)).await?;
        }
        let mut tree = self.reader.by_id(id).await?;
        if let Some(ref sid) = draft.sensor_id {
            self.ensure_sensor_matches_org(sid, tree.organization_id())
                .await?;
        }
        self.ensure_cluster_matches_org(draft.cluster_id, tree.organization_id())
            .await?;
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
        self.ensure_cluster_matches_org(target, tree.organization_id())
            .await?;
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
        self.ensure_sensor_unassigned(&sensor_id, Some(id)).await?;
        let mut tree = self.reader.by_id(id).await?;
        self.ensure_sensor_matches_org(&sensor_id, tree.organization_id())
            .await?;
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

    #[tracing::instrument(level = "debug", skip_all, fields(query.limit = limit))]
    pub async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
        visible: Visibility,
    ) -> Result<Vec<TreeViewWithDistance>, ServiceError> {
        Ok(self
            .reader
            .view_nearest(coord, radius, limit, visible)
            .await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn distinct_planting_years(
        &self,
        visible: Visibility,
    ) -> Result<Vec<PlantingYear>, ServiceError> {
        Ok(self.reader.distinct_planting_years(visible).await?)
    }

    /// Transfers ownership of a clusterless tree to `target`. An attached
    /// sensor is carried along in the same operation (its own organization
    /// coupling would otherwise be violated) — a directly bound sensor must
    /// instead move via its tree, see `SensorService::transfer`.
    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn transfer(
        &self,
        id: Id<Tree>,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let mut tree = self.reader.by_id(id).await?;
        if tree.cluster_id().is_some() {
            return Err(ServiceError::TreeInCluster);
        }
        let mut events = tree.transfer_to(target);
        self.writer.save(&tree).await?;
        if let Some(sensor_id) = tree.sensor_id().cloned() {
            let mut sensor = self.sensor_reader.by_id(&sensor_id).await?;
            events.extend(sensor.transfer_to(target));
            self.sensor_writer.save(&sensor).await?;
        }
        self.event_bus.publish_all(events).await;
        Ok(())
    }
}
