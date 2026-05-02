use std::sync::Arc;

use crate::domain::{
    Id,
    cluster::{
        ClusterAddress, ClusterName, SoilCondition, TreeCluster, TreeClusterDraft,
        TreeClusterReader, TreeClusterSearchQuery, TreeClusterView, TreeClusterWriter,
    },
    events::DomainEvent,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
        provenance::Provenance,
    },
    tree::TreeWriter,
};

use super::{ServiceError, event_bus::EventBus};

pub struct ClusterService {
    reader: Arc<dyn TreeClusterReader>,
    writer: Arc<dyn TreeClusterWriter>,
    tree_writer: Arc<dyn TreeWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl ClusterService {
    pub fn new(
        reader: Arc<dyn TreeClusterReader>,
        writer: Arc<dyn TreeClusterWriter>,
        tree_writer: Arc<dyn TreeWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
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
        let cluster = self.writer.save_new(draft).await?;
        self.event_bus
            .publish(DomainEvent::ClusterTreesChanged {
                cluster_id: cluster.id,
            })
            .await;
        Ok(cluster)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn replace(
        &self,
        id: Id<TreeCluster>,
        name: ClusterName,
        address: ClusterAddress,
        description: String,
        soil_condition: Option<SoilCondition>,
        tree_ids: Vec<Id<crate::domain::tree::Tree>>,
        provenance: Provenance,
    ) -> Result<TreeCluster, ServiceError> {
        let mut cluster = self.reader.by_id(id).await?;
        let trees_changed = cluster.tree_ids != tree_ids;
        cluster.replace_details(name, address, description, soil_condition, provenance);
        cluster.replace_trees(tree_ids);
        self.writer.save(&cluster).await?;
        if trees_changed {
            self.event_bus
                .publish(DomainEvent::ClusterTreesChanged { cluster_id: id })
                .await;
        }
        Ok(cluster)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn delete(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        self.tree_writer.unlink_cluster_id(id).await?;
        self.writer.delete(id).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn archive(&self, id: Id<TreeCluster>) -> Result<(), ServiceError> {
        Ok(self.writer.archive(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(cluster.id = %id))]
    pub async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, ServiceError> {
        Ok(self.reader.center_point(id).await?)
    }
}
