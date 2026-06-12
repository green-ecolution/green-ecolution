use async_trait::async_trait;

use crate::{
    RepositoryError,
    sensor::{
        Sensor, SensorDraft, SensorId, SensorSearchQuery, SensorView,
        data::{SensorReading, SensorReadingDraft, SensorReadingView},
    },
    shared::pagination::{Page, Pagination},
};

#[derive(Debug, Clone)]
pub struct NormalizedValue {
    pub model_ability_id: uuid::Uuid,
    pub value: rust_decimal::Decimal,
}

/// Read-side access to sensors, including aggregate hydration and the
/// HTTP-friendly [`SensorView`] read model.
#[async_trait]
pub trait SensorReader: Send + Sync {
    async fn by_id(&self, id: &SensorId) -> Result<Sensor, RepositoryError>;
    async fn by_ids(&self, ids: &[SensorId]) -> Result<Vec<Sensor>, RepositoryError>;

    /// Returns [`SensorView`] — includes audit timestamps and latest reading.
    async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, RepositoryError>;
    async fn view_by_ids(&self, ids: &[SensorId]) -> Result<Vec<SensorView>, RepositoryError>;
    async fn view_search(
        &self,
        query: SensorSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<SensorView>, RepositoryError>;
}

/// Write-side access to sensors.
#[async_trait]
pub trait SensorWriter: Send + Sync {
    async fn save_new(&self, draft: SensorDraft) -> Result<Sensor, RepositoryError>;
    async fn save(&self, sensor: &Sensor) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &SensorId) -> Result<(), RepositoryError>;
}

/// Read-side access to sensor time-series readings.
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

    /// Per-depth `soil_moisture` values of the sensor's most recent reading.
    async fn latest_volumetric_moisture(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Vec<crate::sensor::data::VolumetricReading>, RepositoryError>;
}

/// Write-side access to sensor readings.
#[async_trait]
pub trait SensorReadingWriter: Send + Sync {
    async fn record(&self, draft: SensorReadingDraft) -> Result<(), RepositoryError>;

    /// Persist a raw reading alongside its normalized per-ability values.
    async fn record_with_normalized(
        &self,
        sensor_id: &SensorId,
        raw: serde_json::Value,
        normalized: &[NormalizedValue],
    ) -> Result<(), RepositoryError>;
}
