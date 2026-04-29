use std::sync::Arc;

use crate::domain::{
    events::DomainEvent,
    sensor::{
        Sensor, SensorCreate, SensorData, SensorQuery, SensorRepository, SensorUpdate,
    },
    shared::pagination::{Page, Pagination},
    tree::TreeRepository,
};

use super::{ServiceError, event_bus::EventBus};

pub struct SensorService {
    sensor_repo: Arc<dyn SensorRepository>,
    tree_repo: Arc<dyn TreeRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl SensorService {
    pub fn new(
        sensor_repo: Arc<dyn SensorRepository>,
        tree_repo: Arc<dyn TreeRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            sensor_repo,
            tree_repo,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn all(
        &self,
        query: SensorQuery,
        pagination: Pagination,
    ) -> Result<Page<Sensor>, ServiceError> {
        Ok(self.sensor_repo.all(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = id))]
    pub async fn by_id(&self, id: &str) -> Result<Sensor, ServiceError> {
        Ok(self.sensor_repo.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[String]) -> Result<Vec<Sensor>, ServiceError> {
        Ok(self.sensor_repo.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, input: SensorCreate) -> Result<Sensor, ServiceError> {
        Ok(self.sensor_repo.create(input).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = id))]
    pub async fn update(
        &self,
        id: &str,
        input: SensorUpdate,
    ) -> Result<Sensor, ServiceError> {
        Ok(self.sensor_repo.update(id, input).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = id))]
    pub async fn delete(&self, id: &str) -> Result<(), ServiceError> {
        self.tree_repo.unlink_sensor_id(id).await?;
        self.sensor_repo.delete(id).await?;
        self.event_bus
            .publish(DomainEvent::SensorDeleted {
                sensor_id: id.to_string(),
                affected_tree_ids: vec![],
            })
            .await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = sensor_id))]
    pub async fn all_data(&self, sensor_id: &str) -> Result<Vec<SensorData>, ServiceError> {
        Ok(self.sensor_repo.all_data(sensor_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = sensor_id))]
    pub async fn latest_data(&self, sensor_id: &str) -> Result<SensorData, ServiceError> {
        Ok(self.sensor_repo.latest_data(sensor_id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %data.sensor_id))]
    pub async fn create_data(&self, data: SensorData) -> Result<(), ServiceError> {
        let sensor_id = data.sensor_id.clone();
        self.sensor_repo.create_data(data).await?;
        self.event_bus
            .publish(DomainEvent::SensorDataReceived {
                sensor_id,
                data: serde_json::Value::Null,
            })
            .await;
        Ok(())
    }
}
