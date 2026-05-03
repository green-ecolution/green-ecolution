use std::sync::Arc;

use crate::domain::{
    events::DomainEvent,
    sensor::{
        Sensor, SensorDraft, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorStatus, SensorView, SensorWriter,
        data::{SensorReadingDraft, SensorReadingView},
    },
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
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
    pub async fn move_to(&self, id: &SensorId, new: Coordinate) -> Result<Sensor, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        sensor.move_to(new);
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

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %draft.sensor_id))]
    pub async fn record_reading(&self, draft: SensorReadingDraft) -> Result<(), ServiceError> {
        let sensor_id = draft.sensor_id.clone();
        let data = draft.data.clone();
        self.reading_writer.record(draft).await?;
        self.event_bus
            .publish(DomainEvent::SensorDataReceived {
                sensor_id,
                ts: chrono::Utc::now(),
                data,
            })
            .await;
        Ok(())
    }
}
