use std::sync::Arc;

use domain::{
    Id,
    authorization::Visibility,
    organization::Organization,
    sensor::SensorId,
    shared::{coordinates::Coordinate, distance::Distance, pagination::Pagination},
};
use irrigation_domain::{
    cluster::{TreeCluster, TreeClusterReader},
    events::IrrigationEvent,
    tree::{
        PlantingYear, Tree, TreeDraft, TreeMarker, TreeReader, TreeSearchPage, TreeSearchQuery,
        TreeView, TreeViewWithDistance, TreeWriter,
    },
    watering_status::WateringStatus,
};

use crate::error::IrrigationError;
use server::service::{ServiceError, event_bus::EventBus, sensor_service::SensorService};

/// The single tree operation another service needs from this one.
///
/// Deleting a tree publishes `TreeDeleted` and lets the cluster handlers
/// recompute centroid and status, so a second caller must not reimplement it.
/// The port stays narrow on purpose: a consumer depends on the one method it
/// uses instead of on the whole `TreeService`, which keeps the service graph
/// readable and its fakes to one method. `PluginIngestService` is the only
/// consumer today.
#[async_trait::async_trait]
pub trait TreeDeletion: Send + Sync {
    async fn delete(&self, id: Id<Tree>) -> Result<(), ServiceError>;
}

#[async_trait::async_trait]
impl TreeDeletion for TreeService {
    async fn delete(&self, id: Id<Tree>) -> Result<(), ServiceError> {
        Ok(TreeService::delete(self, id).await?)
    }
}

pub struct TreeService {
    reader: Arc<dyn TreeReader>,
    writer: Arc<dyn TreeWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    sensors: Arc<SensorService>,
    event_bus: Arc<dyn EventBus>,
}

impl TreeService {
    pub fn new(
        reader: Arc<dyn TreeReader>,
        writer: Arc<dyn TreeWriter>,
        cluster_reader: Arc<dyn TreeClusterReader>,
        sensors: Arc<SensorService>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            cluster_reader,
            sensors,
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
    ) -> Result<(), IrrigationError> {
        if let Some(cid) = cluster_id {
            let cluster = self.cluster_reader.by_id(cid).await?;
            if cluster.organization_id() != org {
                return Err(IrrigationError::ClusterVsTree);
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: TreeSearchQuery,
        pagination: Pagination,
    ) -> Result<TreeSearchPage, IrrigationError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_markers(
        &self,
        query: TreeSearchQuery,
    ) -> Result<Vec<TreeMarker>, IrrigationError> {
        Ok(self.reader.view_markers(query).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn by_id(&self, id: Id<Tree>) -> Result<Tree, IrrigationError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, IrrigationError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn view_by_id(&self, id: Id<Tree>) -> Result<TreeView, IrrigationError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<TreeView>, IrrigationError> {
        Ok(self.reader.view_by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %sensor_id))]
    pub async fn view_by_sensor_id(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Option<TreeView>, IrrigationError> {
        Ok(self.reader.view_by_sensor_id(sensor_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: TreeDraft) -> Result<Tree, IrrigationError> {
        self.ensure_cluster_matches_org(draft.cluster_id, draft.organization_id)
            .await?;
        let tree = self.writer.save_new(draft).await?;
        self.event_bus
            .publish(
                IrrigationEvent::TreeCreated {
                    tree_id: tree.id,
                    cluster_id: tree.cluster_id(),
                }
                .into_domain(),
            )
            .await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn replace(&self, id: Id<Tree>, draft: TreeDraft) -> Result<Tree, IrrigationError> {
        let mut tree = self.reader.by_id(id).await?;
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
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn move_to_cluster(
        &self,
        id: Id<Tree>,
        target: Option<Id<TreeCluster>>,
    ) -> Result<Tree, IrrigationError> {
        let mut tree = self.reader.by_id(id).await?;
        self.ensure_cluster_matches_org(target, tree.organization_id())
            .await?;
        let events = tree.move_to_cluster(target);
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn record_watering_status(
        &self,
        id: Id<Tree>,
        status: WateringStatus,
    ) -> Result<Tree, IrrigationError> {
        let mut tree = self.reader.by_id(id).await?;
        let events = tree.record_watering_status(status);
        self.writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(tree)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(tree.id = %id))]
    pub async fn delete(&self, id: Id<Tree>) -> Result<(), IrrigationError> {
        let tree = self.reader.by_id(id).await?;
        let cluster_id = tree.cluster_id();
        self.writer.delete(id).await?;
        self.event_bus
            .publish(
                IrrigationEvent::TreeDeleted {
                    tree_id: id,
                    cluster_id,
                }
                .into_domain(),
            )
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
    ) -> Result<Vec<TreeViewWithDistance>, IrrigationError> {
        Ok(self
            .reader
            .view_nearest(coord, radius, limit, visible)
            .await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn distinct_planting_years(
        &self,
        visible: Visibility,
    ) -> Result<Vec<PlantingYear>, IrrigationError> {
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
    ) -> Result<(), IrrigationError> {
        let mut tree = self.reader.by_id(id).await?;
        if tree.cluster_id().is_some() {
            return Err(IrrigationError::TreeInCluster);
        }
        let events = tree.transfer_to(target);
        self.writer.save(&tree).await?;
        if let Some(sensor_id) = self.reader.sensor_ids(&[tree.id]).await?.remove(&tree.id) {
            self.sensors
                .transfer_with_target(&sensor_id, target)
                .await?;
        }
        self.event_bus.publish_all(events).await;
        Ok(())
    }
}
