use async_trait::async_trait;

use crate::domain::{
    Id, RepositoryError,
    cluster::{TreeCluster, TreeClusterDraft, TreeClusterSearchQuery, TreeClusterView},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

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

    async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, RepositoryError>;
}

#[async_trait]
pub trait TreeClusterWriter: Send + Sync {
    async fn save_new(&self, draft: TreeClusterDraft) -> Result<TreeCluster, RepositoryError>;
    async fn save(&self, cluster: &TreeCluster) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
}
