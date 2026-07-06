use std::sync::Arc;

use domain::{
    Id,
    cluster::{
        ClusterBoundaryView, ClusterMarker, ClusterStatistics, TreeCluster, TreeClusterDraft,
        TreeClusterReader, TreeClusterSearchQuery, TreeClusterUpdate, TreeClusterView,
        TreeClusterWriter,
    },
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    tree::{Tree, TreeReader, TreeWriter},
};

use super::{ServiceError, event_bus::EventBus};

pub struct ClusterService {
    reader: Arc<dyn TreeClusterReader>,
    writer: Arc<dyn TreeClusterWriter>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl ClusterService {
    pub fn new(
        reader: Arc<dyn TreeClusterReader>,
        writer: Arc<dyn TreeClusterWriter>,
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            tree_reader,
            tree_writer,
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
    pub async fn view_markers(&self) -> Result<Vec<ClusterMarker>, ServiceError> {
        Ok(self.reader.view_markers().await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn boundaries(&self) -> Result<Vec<ClusterBoundaryView>, ServiceError> {
        Ok(self.reader.boundaries().await?)
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: TreeClusterDraft) -> Result<TreeCluster, ServiceError> {
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
    pub async fn statistics(&self) -> Result<ClusterStatistics, ServiceError> {
        Ok(self.reader.statistics().await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, ServiceError> {
        Ok(self.reader.center_point(id).await?)
    }
}
