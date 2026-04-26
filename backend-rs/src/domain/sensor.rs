use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    shared::{coordinates::Coordinate, pagination::{Page, Pagination}, provider_info::ProviderInfo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Option<SensorStatus>,
    pub latest_data: Option<SensorData>,
    pub coordinates: Coordinate,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Clone, Copy)]
pub struct Watermark {
    pub depth: i32,
    pub resistance: i32,
    pub centibar: i32,
}

#[derive(Debug, Clone)]
pub struct SensorData {
    pub id: Id<Self>,
    pub sensor_id: Id<Sensor>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub battery: f64,
    pub humidity: f64,
    pub temperature: f64,
    pub watermarks: Vec<Watermark>,
}

#[derive(Debug)]
pub struct SensorCreate {
    pub id: Id<Sensor>,
    pub status: SensorStatus,
    pub latest_data: Option<SensorData>,
    pub coordinate: Coordinate,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct SensorUpdate {
    pub status: Option<SensorStatus>,
    pub latest_data: Option<SensorData>,
    pub coordinate: Option<Coordinate>,
    pub provider_info: Option<ProviderInfo>,
}

#[derive(Debug, Default)]
pub struct SensorQuery {
    pub provider: Option<String>,
}

#[async_trait::async_trait]
pub trait SensorRepository: Send + Sync {
    async fn all(&self, query: SensorQuery, pagination: Pagination) -> Result<Page<Sensor>, RepositoryError>;
    async fn count(&self, query: SensorQuery) -> Result<u64, RepositoryError>;
    async fn by_id(&self, id: Id<Sensor>) -> Result<Sensor, RepositoryError>;
    async fn create(&self, entity: SensorCreate) -> Result<Sensor, RepositoryError>;
    async fn update(&self, id: Id<Sensor>, entity: SensorUpdate)
    -> Result<Sensor, RepositoryError>;
    async fn delete(&self, id: Id<Sensor>) -> Result<(), RepositoryError>;

    async fn all_data(&self, id: Id<Sensor>) -> Result<Page<SensorData>, RepositoryError>;
    async fn latest_data(&self, id: Id<Sensor>) -> Result<SensorData, RepositoryError>;
    async fn create_data(&self, data: SensorData) -> Result<(), RepositoryError>;
}
