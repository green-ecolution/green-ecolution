use async_trait::async_trait;

use crate::{
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

/// Read-side access to trees, including aggregate hydration and the
/// HTTP-friendly [`TreeView`] read model.
#[async_trait]
pub trait TreeReader: Send + Sync {
    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError>;

    /// Loads the [`Tree`] (aggregate) bound to `sensor_id`, if any.
    /// Returns `Ok(None)` if no tree is currently linked to that sensor.
    async fn by_sensor_id(&self, sensor_id: &SensorId) -> Result<Option<Tree>, RepositoryError>;

    /// Loads all trees that belong to `cluster_id`. Empty vec if the cluster
    /// has no trees.
    async fn by_cluster_id(
        &self,
        cluster_id: Id<TreeCluster>,
    ) -> Result<Vec<Tree>, RepositoryError>;

    /// Returns [`TreeView`] — includes audit timestamps.
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
    /// Returns trees within `radius` of `coord`, ordered by distance ascending,
    /// up to `limit` results.
    async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeViewWithDistance>, RepositoryError>;

    /// Returns the [`Tree`] aggregate closest to `coord` within `radius`, or
    /// `Ok(None)` if no tree is within range. Used by the sensor ingest path
    /// to auto-link a freshly-arrived sensor to the nearest tree.
    async fn find_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
    ) -> Result<Option<Tree>, RepositoryError>;

    async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, RepositoryError>;
}

/// Write-side access to trees.
#[async_trait]
pub trait TreeWriter: Send + Sync {
    async fn save_new(&self, draft: TreeDraft) -> Result<Tree, RepositoryError>;
    async fn save(&self, tree: &Tree) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError>;
}
