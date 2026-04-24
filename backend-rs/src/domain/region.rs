use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    shared::{coordinates::Coordinate, pagination::Page},
};

#[derive(Debug, Clone)]
pub struct Region {
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
}

impl Region {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        name: String,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            name,
        }
    }

    pub fn id(&self) -> &Id<Self> {
        &self.id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct RegionCreate {
    pub name: String,
}

pub struct RegionUpdate {
    pub name: String,
}

#[trait_variant::make(Send)]
pub trait RegionRepository {
    async fn all(&self) -> Result<Page<Region>, RepositoryError>;
    async fn by_id(&self, id: Id<Region>) -> Result<Region, RepositoryError>;
    async fn by_point(&self, coord: Coordinate) -> Result<Region, RepositoryError>;
    async fn create(&self, entity: RegionCreate) -> Result<Region, RepositoryError>;
    async fn update(&self, id: Id<Region>, entity: RegionUpdate)
    -> Result<Region, RepositoryError>;
    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError>;
}
