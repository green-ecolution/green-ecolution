use chrono::{DateTime, Utc};

use crate::domain::{
    RepositoryError,
    shared::{coordinates::Coordinate, pagination::{Page, Pagination}, provider_info::ProviderInfo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "sensor_status", rename_all = "snake_case")]
pub enum SensorStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SensorStatus,
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
    pub id: i32,
    pub sensor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug)]
pub struct SensorCreate {
    pub id: String,
    pub status: SensorStatus,
    pub coordinate: Coordinate,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct SensorUpdate {
    pub status: Option<SensorStatus>,
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
    async fn by_id(&self, id: &str) -> Result<Sensor, RepositoryError>;
    async fn by_ids(&self, ids: &[String]) -> Result<Vec<Sensor>, RepositoryError>;
    async fn create(&self, entity: SensorCreate) -> Result<Sensor, RepositoryError>;
    async fn update(&self, id: &str, entity: SensorUpdate) -> Result<Sensor, RepositoryError>;
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;

    async fn all_data(&self, sensor_id: &str) -> Result<Vec<SensorData>, RepositoryError>;
    async fn latest_data(&self, sensor_id: &str) -> Result<SensorData, RepositoryError>;
    async fn create_data(&self, data: SensorData) -> Result<(), RepositoryError>;
}
