use async_trait::async_trait;

use crate::domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    sensor::SensorId,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
    },
    tree::{PlantingYear, Tree, TreeDraft, TreeSearchQuery, TreeView, TreeViewWithDistance},
};

#[async_trait]
pub trait TreeReader: Send + Sync {
    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError>;

    async fn view_by_id(&self, id: Id<Tree>) -> Result<TreeView, RepositoryError>;
    async fn view_by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<TreeView>, RepositoryError>;
    async fn view_by_sensor_id(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Option<TreeView>, RepositoryError>;
    async fn view_search(
        &self,
        query: TreeSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeView>, RepositoryError>;
    async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeViewWithDistance>, RepositoryError>;

    async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, RepositoryError>;
}

#[async_trait]
pub trait TreeWriter: Send + Sync {
    async fn save_new(&self, draft: TreeDraft) -> Result<Tree, RepositoryError>;
    async fn save(&self, tree: &Tree) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError>;
    async fn unlink_cluster_id(&self, cluster_id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn unlink_sensor_id(&self, sensor_id: &SensorId) -> Result<(), RepositoryError>;
}
