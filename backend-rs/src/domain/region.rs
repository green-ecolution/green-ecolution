use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

#[derive(Debug, Clone)]
pub struct Region {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct RegionCreate {
    pub name: String,
}

pub struct RegionUpdate {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct RegionQuery {
    pub provider: Option<String>,
}

#[async_trait::async_trait]
pub trait RegionRepository: Send + Sync {
    async fn all(&self, query: RegionQuery, pagination: Pagination) -> Result<Page<Region>, RepositoryError>;
    async fn by_id(&self, id: Id<Region>) -> Result<Region, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, RepositoryError>;
    async fn by_point(&self, coord: Coordinate) -> Result<Region, RepositoryError>;
    async fn create(&self, entity: RegionCreate) -> Result<Region, RepositoryError>;
    async fn update(&self, id: Id<Region>, entity: RegionUpdate)
    -> Result<Region, RepositoryError>;
    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError>;
}
