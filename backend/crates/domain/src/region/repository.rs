use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    region::{Region, RegionDraft, RegionName, RegionSearchQuery},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

/// Read-side access to regions.
#[async_trait]
pub trait RegionReader: Send + Sync {
    async fn by_id(&self, id: Id<Region>) -> Result<Region, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, RepositoryError>;
    async fn by_name(&self, name: &RegionName) -> Result<Option<Region>, RepositoryError>;
    /// Returns the region whose polygon contains `coord`, if any.
    async fn by_point(&self, coord: Coordinate) -> Result<Option<Region>, RepositoryError>;
    async fn search(
        &self,
        query: RegionSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<Region>, RepositoryError>;
}

/// Write-side access to regions.
#[async_trait]
pub trait RegionWriter: Send + Sync {
    async fn save_new(&self, draft: RegionDraft) -> Result<Region, RepositoryError>;
    async fn save(&self, region: &Region) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError>;
}
