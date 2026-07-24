use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::{
    Id,
    authorization::Visibility,
    cluster::{
        ClusterBoundaryView, ClusterMarker, ClusterStatistics, SoilMoistureBucket,
        SoilMoistureOverview, TreeCluster, TreeClusterDraft, TreeClusterReader,
        TreeClusterSearchQuery, TreeClusterUpdate, TreeClusterView, TreeClusterWriter,
        condition_series,
    },
    events::DomainEvent,
    organization::Organization,
    sensor::{SensorReader, SensorWriter},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    tree::{Tree, TreeReader, TreeWriter, volumetric_thresholds},
};

use super::{ServiceError, event_bus::EventBus};

pub struct ClusterService {
    reader: Arc<dyn TreeClusterReader>,
    writer: Arc<dyn TreeClusterWriter>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    sensor_reader: Arc<dyn SensorReader>,
    sensor_writer: Arc<dyn SensorWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl ClusterService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reader: Arc<dyn TreeClusterReader>,
        writer: Arc<dyn TreeClusterWriter>,
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        sensor_reader: Arc<dyn SensorReader>,
        sensor_writer: Arc<dyn SensorWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            tree_reader,
            tree_writer,
            sensor_reader,
            sensor_writer,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: TreeClusterSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeClusterView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_markers(
        &self,
        visible: Visibility,
    ) -> Result<Vec<ClusterMarker>, ServiceError> {
        Ok(self.reader.view_markers(visible).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn boundaries(
        &self,
        visible: Visibility,
    ) -> Result<Vec<ClusterBoundaryView>, ServiceError> {
        Ok(self.reader.boundaries(visible).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<TreeCluster>]) -> Result<Vec<TreeCluster>, ServiceError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn view_by_id(&self, id: Id<TreeCluster>) -> Result<TreeClusterView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    /// Rejects a cluster whose tree membership crosses an organization
    /// boundary — every member tree must belong to the same org as the
    /// cluster itself.
    async fn ensure_trees_match_org(
        &self,
        tree_ids: &[Id<Tree>],
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        if tree_ids.is_empty() {
            return Ok(());
        }
        let trees = self.tree_reader.by_ids(tree_ids).await?;
        if trees.iter().any(|t| t.organization_id() != org) {
            return Err(ServiceError::OrganizationMismatch);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: TreeClusterDraft) -> Result<TreeCluster, ServiceError> {
        self.ensure_trees_match_org(&draft.tree_ids, draft.organization_id)
            .await?;
        let source_events = self.source_cluster_events(&draft.tree_ids, None).await?;
        let cluster = self.writer.save_new(draft).await?;
        let mut events = vec![DomainEvent::ClusterTreesChanged {
            cluster_id: cluster.id,
        }];
        events.extend(source_events);
        self.event_bus.publish_all(events).await;
        Ok(cluster)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn replace(
        &self,
        id: Id<TreeCluster>,
        update: TreeClusterUpdate,
    ) -> Result<TreeCluster, ServiceError> {
        let mut cluster = self.reader.by_id(id).await?;
        self.ensure_trees_match_org(&update.tree_ids, cluster.organization_id())
            .await?;
        let source_events = self
            .source_cluster_events(&update.tree_ids, Some(id))
            .await?;
        let mut events = cluster.replace_details(
            update.name,
            update.address,
            update.description,
            update.soil_condition,
            update.provenance,
        );
        events.extend(cluster.replace_trees(update.tree_ids));
        self.writer.save(&cluster).await?;
        events.extend(source_events);
        self.event_bus.publish_all(events).await;
        Ok(cluster)
    }

    /// Assigning trees to a cluster implicitly pulls them out of the cluster
    /// they currently belong to (the repo save rewrites `tree_cluster_id`).
    /// Those source clusters need a `ClusterTreesChanged` too, or their
    /// centroid, region, and watering status stay stale.
    async fn source_cluster_events(
        &self,
        tree_ids: &[Id<Tree>],
        target: Option<Id<TreeCluster>>,
    ) -> Result<Vec<DomainEvent>, ServiceError> {
        if tree_ids.is_empty() {
            return Ok(Vec::new());
        }
        let trees = self.tree_reader.by_ids(tree_ids).await?;
        let sources: std::collections::HashSet<Id<TreeCluster>> = trees
            .iter()
            .filter_map(|t| t.cluster_id())
            .filter(|cid| Some(*cid) != target)
            .collect();
        Ok(sources
            .into_iter()
            .map(|cluster_id| DomainEvent::ClusterTreesChanged { cluster_id })
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn delete(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        let trees = self.tree_reader.by_cluster_id(id).await?;
        let mut events = Vec::new();
        for mut tree in trees {
            events.extend(tree.move_to_cluster(None));
            self.tree_writer.save(&tree).await?;
        }
        self.writer.delete(id).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn archive(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        Ok(self.writer.archive(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn statistics(&self, visible: Visibility) -> Result<ClusterStatistics, ServiceError> {
        Ok(self.reader.statistics(visible).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, ServiceError> {
        Ok(self.reader.center_point(id).await?)
    }

    /// Thresholds are derived from the cluster's KA5 soil type; an unknown or
    /// unset soil yields an empty threshold list, not an error.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn soil_moisture_overview(
        &self,
        id: Id<TreeCluster>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<SoilMoistureOverview, ServiceError> {
        let view = self.reader.view_by_id(id).await?;
        let series = self
            .reader
            .soil_moisture_series(id, from, to, bucket)
            .await?;
        let thresholds = view
            .soil_condition
            .map(|soil| {
                series
                    .iter()
                    .filter_map(|s| volumetric_thresholds(soil, s.depth_cm))
                    .collect()
            })
            .unwrap_or_default();
        let condition = view
            .soil_condition
            .map(|soil| condition_series(&series, soil))
            .unwrap_or_default();
        let watering_events = self.reader.watering_events(id).await?;
        Ok(SoilMoistureOverview {
            bucket,
            series,
            thresholds,
            condition,
            watering_events,
        })
    }

    /// Transfers ownership of the cluster, its member trees, and their
    /// attached sensors to `target` in one operation.
    ///
    /// No distributed rollback: saves happen sequentially, so an error midway
    /// through the tree loop leaves a partially transferred cluster. Accepted
    /// for v1 — transfer is an admin operation and idempotent to retry.
    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn transfer(
        &self,
        id: Id<TreeCluster>,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let mut cluster = self.reader.by_id(id).await?;
        let mut events = cluster.transfer_to(target);
        self.writer.save(&cluster).await?;
        for tree_id in cluster.tree_ids.clone() {
            let mut tree = self.tree_reader.by_id(tree_id).await?;
            events.extend(tree.transfer_to(target));
            self.tree_writer.save(&tree).await?;
            if let Some(sensor_id) = tree.sensor_id().cloned() {
                let mut sensor = self.sensor_reader.by_id(&sensor_id).await?;
                events.extend(sensor.transfer_to(target));
                self.sensor_writer.save(&sensor).await?;
            }
        }
        self.event_bus.publish_all(events).await;
        Ok(())
    }
}
