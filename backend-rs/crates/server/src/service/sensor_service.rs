use std::sync::Arc;

use domain::{
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    sensor::{
        Sensor, SensorDraft, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorStatus, SensorView, SensorWriter,
        data::{MqttPayload, SensorReadingDraft, SensorReadingView},
    },
    shared::pagination::{Page, Pagination},
    tree::{TreeReader, TreeWriter},
};

use super::{ServiceError, event_bus::EventBus};

pub struct SensorService {
    reader: Arc<dyn SensorReader>,
    writer: Arc<dyn SensorWriter>,
    reading_reader: Arc<dyn SensorReadingReader>,
    reading_writer: Arc<dyn SensorReadingWriter>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl SensorService {
    pub fn new(
        reader: Arc<dyn SensorReader>,
        writer: Arc<dyn SensorWriter>,
        reading_reader: Arc<dyn SensorReadingReader>,
        reading_writer: Arc<dyn SensorReadingWriter>,
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            reading_reader,
            reading_writer,
            tree_reader,
            tree_writer,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: SensorSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<SensorView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn by_id(&self, id: &SensorId) -> Result<Sensor, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[SensorId]) -> Result<Vec<Sensor>, ServiceError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_by_ids(&self, ids: &[SensorId]) -> Result<Vec<SensorView>, ServiceError> {
        Ok(self.reader.view_by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: SensorDraft) -> Result<Sensor, ServiceError> {
        Ok(self.writer.save_new(draft).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn change_status(
        &self,
        id: &SensorId,
        new: SensorStatus,
    ) -> Result<Sensor, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        sensor.change_status(new);
        self.writer.save(&sensor).await?;
        Ok(sensor)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn delete(&self, id: &SensorId) -> Result<(), ServiceError> {
        let mut events = Vec::new();
        if let Some(mut tree) = self.tree_reader.by_sensor_id(id).await? {
            events.extend(tree.detach_sensor());
            self.tree_writer.save(&tree).await?;
        }
        self.writer.delete(id).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %sensor_id))]
    pub async fn view_history(
        &self,
        sensor_id: &SensorId,
        limit: i64,
    ) -> Result<Vec<SensorReadingView>, ServiceError> {
        Ok(self.reading_reader.view_history(sensor_id, limit).await?)
    }

    /// Ingests one MQTT uplink message: bump the sensor's status to `Online`
    /// if needed, persist the raw reading, and publish
    /// [`DomainEvent::SensorDataReceived`] for subscribers. The sensor must
    /// already exist (registration is an explicit step in the new schema).
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %payload.device))]
    pub async fn handle_message(&self, payload: MqttPayload) -> Result<(), ServiceError> {
        let sensor_id = SensorId::new(payload.device.clone())?;
        let mut sensor = self.reader.by_id(&sensor_id).await?;
        self.bump_online(&mut sensor).await?;
        self.record_reading(sensor.id, payload).await
    }

    async fn bump_online(&self, sensor: &mut Sensor) -> Result<(), ServiceError> {
        if sensor.status() != SensorStatus::Online {
            sensor.change_status(SensorStatus::Online);
            self.writer.save(sensor).await?;
        }
        Ok(())
    }

    async fn record_reading(
        &self,
        sensor_id: SensorId,
        payload: MqttPayload,
    ) -> Result<(), ServiceError> {
        let data = serde_json::to_value(&payload).map_err(|e| {
            ServiceError::Repository(domain::RepositoryError::Internal(format!(
                "failed to serialise mqtt payload: {e}"
            )))
        })?;
        let watermarks = payload.watermarks;

        self.reading_writer
            .record(SensorReadingDraft {
                sensor_id: sensor_id.clone(),
                data,
            })
            .await?;

        self.event_bus
            .publish(DomainEvent::SensorDataReceived(SensorDataReceivedPayload {
                sensor_id,
                readings: SensorReadings::Watermarks(watermarks),
            }))
            .await;

        Ok(())
    }
}
