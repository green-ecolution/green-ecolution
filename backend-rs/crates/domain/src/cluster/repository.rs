use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    cluster::{TreeCluster, TreeClusterDraft, TreeClusterSearchQuery, TreeClusterView},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

/// Read-side access to tree clusters, including aggregate hydration and the
/// HTTP-friendly [`TreeClusterView`] read model.
#[async_trait]
pub trait TreeClusterReader: Send + Sync {
    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<TreeCluster>]) -> Result<Vec<TreeCluster>, RepositoryError>;

    async fn view_by_id(&self, id: Id<TreeCluster>) -> Result<TreeClusterView, RepositoryError>;
    async fn view_by_ids(
        &self,
        ids: &[Id<TreeCluster>],
    ) -> Result<Vec<TreeClusterView>, RepositoryError>;
    async fn view_search(
        &self,
        query: TreeClusterSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeClusterView>, RepositoryError>;

    /// Returns the DB-persisted centroid for a cluster, or `None` if the
    /// cluster currently has no trees.
    async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, RepositoryError>;
}

/// Write-side access to tree clusters.
#[async_trait]
pub trait TreeClusterWriter: Send + Sync {
    async fn save_new(&self, draft: TreeClusterDraft) -> Result<TreeCluster, RepositoryError>;
    async fn save(&self, cluster: &TreeCluster) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
}
