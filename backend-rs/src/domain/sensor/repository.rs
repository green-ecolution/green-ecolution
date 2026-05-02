use async_trait::async_trait;

use crate::domain::{
    RepositoryError,
    sensor::{
        Sensor, SensorDraft, SensorId, SensorSearchQuery, SensorView,
        data::{SensorReading, SensorReadingDraft, SensorReadingView},
    },
    shared::pagination::{Page, Pagination},
};

#[async_trait]
pub trait SensorReader: Send + Sync {
    async fn by_id(&self, id: &SensorId) -> Result<Sensor, RepositoryError>;
    async fn by_ids(&self, ids: &[SensorId]) -> Result<Vec<Sensor>, RepositoryError>;

    async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, RepositoryError>;
    async fn view_by_ids(&self, ids: &[SensorId]) -> Result<Vec<SensorView>, RepositoryError>;
    async fn view_search(
        &self,
        query: SensorSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<SensorView>, RepositoryError>;
}

#[async_trait]
pub trait SensorWriter: Send + Sync {
    async fn save_new(&self, draft: SensorDraft) -> Result<Sensor, RepositoryError>;
    async fn save(&self, sensor: &Sensor) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &SensorId) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SensorReadingReader: Send + Sync {
    async fn history(
        &self,
        sensor_id: &SensorId,
        limit: i64,
    ) -> Result<Vec<SensorReading>, RepositoryError>;
    async fn latest(&self, sensor_id: &SensorId) -> Result<Option<SensorReading>, RepositoryError>;

    async fn view_history(
        &self,
        sensor_id: &SensorId,
        limit: i64,
    ) -> Result<Vec<SensorReadingView>, RepositoryError>;
}

#[async_trait]
pub trait SensorReadingWriter: Send + Sync {
    async fn record(&self, draft: SensorReadingDraft) -> Result<(), RepositoryError>;
}
