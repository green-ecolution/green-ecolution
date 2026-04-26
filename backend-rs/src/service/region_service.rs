use std::sync::Arc;

use crate::domain::{
    Id,
    region::{Region, RegionCreate, RegionQuery, RegionRepository, RegionUpdate},
    shared::coordinates::Coordinate,
    shared::pagination::{Page, Pagination},
};

use super::ServiceError;

pub struct RegionService {
    region_repo: Arc<dyn RegionRepository>,
}

impl RegionService {
    pub fn new(region_repo: Arc<dyn RegionRepository>) -> Self {
        Self { region_repo }
    }

    pub async fn all(
        &self,
        query: RegionQuery,
        pagination: Pagination,
    ) -> Result<Page<Region>, ServiceError> {
        Ok(self.region_repo.all(query, pagination).await?)
    }

    pub async fn by_id(&self, id: Id<Region>) -> Result<Region, ServiceError> {
        Ok(self.region_repo.by_id(id).await?)
    }

    pub async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, ServiceError> {
        Ok(self.region_repo.by_ids(ids).await?)
    }

    pub async fn by_point(&self, coord: Coordinate) -> Result<Region, ServiceError> {
        Ok(self.region_repo.by_point(coord).await?)
    }

    pub async fn create(&self, entity: RegionCreate) -> Result<Region, ServiceError> {
        Ok(self.region_repo.create(entity).await?)
    }

    pub async fn update(
        &self,
        id: Id<Region>,
        entity: RegionUpdate,
    ) -> Result<Region, ServiceError> {
        Ok(self.region_repo.update(id, entity).await?)
    }

    pub async fn delete(&self, id: Id<Region>) -> Result<(), ServiceError> {
        Ok(self.region_repo.delete(id).await?)
    }
}
