use std::sync::Arc;

use crate::domain::{
    Id,
    region::{Region, RegionDraft, RegionName, RegionReader, RegionSearchQuery, RegionWriter},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

use super::ServiceError;

pub struct RegionService {
    reader: Arc<dyn RegionReader>,
    writer: Arc<dyn RegionWriter>,
}

impl RegionService {
    pub fn new(reader: Arc<dyn RegionReader>, writer: Arc<dyn RegionWriter>) -> Self {
        Self { reader, writer }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search(
        &self,
        query: RegionSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<Region>, ServiceError> {
        Ok(self.reader.search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(region.id = %id))]
    pub async fn by_id(&self, id: Id<Region>) -> Result<Region, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, ServiceError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_point(&self, coord: Coordinate) -> Result<Option<Region>, ServiceError> {
        Ok(self.reader.by_point(coord).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: RegionDraft) -> Result<Region, ServiceError> {
        Ok(self.writer.save_new(draft).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(region.id = %id))]
    pub async fn rename(
        &self,
        id: Id<Region>,
        new_name: RegionName,
    ) -> Result<Region, ServiceError> {
        let mut region = self.reader.by_id(id).await?;
        region.rename(new_name);
        self.writer.save(&region).await?;
        Ok(region)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(region.id = %id))]
    pub async fn delete(&self, id: Id<Region>) -> Result<(), ServiceError> {
        Ok(self.writer.delete(id).await?)
    }
}
