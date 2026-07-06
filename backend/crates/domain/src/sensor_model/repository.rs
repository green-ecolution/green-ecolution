use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    sensor_model::{SensorModel, SensorModelName},
};

#[async_trait]
pub trait SensorModelReader: Send + Sync {
    async fn list(&self) -> Result<Vec<SensorModel>, RepositoryError>;
    async fn by_id(&self, id: Id<SensorModel>) -> Result<SensorModel, RepositoryError>;
    async fn by_name(&self, name: &SensorModelName) -> Result<SensorModel, RepositoryError>;
}
