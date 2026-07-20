use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    Id, RepositoryError,
    cluster::{
        ClusterBoundaryView, ClusterMarker, ClusterStatistics, ClusterWateringEvent,
        SoilMoistureBucket, SoilMoistureDepthSeries, TreeCluster, TreeClusterDraft,
        TreeClusterSearchQuery, TreeClusterView,
    },
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

    /// Returns marker-projected clusters that have a centroid.
    /// Archived clusters and clusters without trees are excluded.
    async fn view_markers(&self) -> Result<Vec<ClusterMarker>, RepositoryError>;

    /// Returns one convex-hull boundary polygon (GeoJSON, buffered in meters)
    /// per non-archived cluster that has at least one geo-located tree.
    /// Clusters without trees are omitted.
    async fn boundaries(&self) -> Result<Vec<ClusterBoundaryView>, RepositoryError>;

    /// Returns the DB-persisted centroid for a cluster, or `None` if the
    /// cluster currently has no trees.
    async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, RepositoryError>;

    async fn statistics(&self) -> Result<ClusterStatistics, RepositoryError>;

    /// Bucketed volumetric soil-moisture readings (mean/min/max per depth)
    /// from all sensors currently linked to the cluster's trees. Readings
    /// outside 0–100 % are sensor sentinels and are excluded.
    async fn soil_moisture_series(
        &self,
        id: Id<TreeCluster>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<Vec<SoilMoistureDepthSeries>, RepositoryError>;

    /// Finished watering-plan runs that included this cluster, newest first.
    async fn watering_events(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Vec<ClusterWateringEvent>, RepositoryError>;
}

/// Write-side access to tree clusters.
#[async_trait]
pub trait TreeClusterWriter: Send + Sync {
    async fn save_new(&self, draft: TreeClusterDraft) -> Result<TreeCluster, RepositoryError>;
    async fn save(&self, cluster: &TreeCluster) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
}
