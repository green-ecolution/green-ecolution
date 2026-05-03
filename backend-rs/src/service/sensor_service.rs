use std::sync::Arc;

use crate::domain::{
    RepositoryError,
    events::DomainEvent,
    sensor::{
        Sensor, SensorDraft, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorStatus, SensorView, SensorWriter,
        data::{MqttPayload, SensorReadingDraft, SensorReadingView},
    },
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        provenance::Provenance,
    },
    tree::{TreeReader, TreeWriter},
};

/// Radius (metres) within which an auto-created sensor is linked to the
/// nearest tree. Mirrors the Go backend's `FindNearestTree` query.
const AUTO_LINK_RADIUS_M: f64 = 3.0;

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

    /// Ingests one MQTT uplink message: get-or-create the sensor, persist the
    /// raw reading, link the sensor to the nearest tree on first sight, and
    /// publish a typed [`DomainEvent::SensorDataReceived`] for subscribers.
    ///
    /// Idempotent on the sensor record: subsequent messages from a known
    /// device update its coordinates and bump status to `Online`. The link to
    /// a tree is only attempted when the sensor is first created.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %payload.device))]
    pub async fn handle_message(&self, payload: MqttPayload) -> Result<(), ServiceError> {
        let sensor_id = SensorId::new(payload.device.clone())
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        let coord = Coordinate::new(payload.latitude, payload.longitude)
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;

        let sensor = match self.reader.by_id(&sensor_id).await {
            Ok(mut existing) => {
                let mut changed = false;
                if existing.coordinate != coord {
                    existing.move_to(coord);
                    changed = true;
                }
                if existing.status != SensorStatus::Online {
                    existing.change_status(SensorStatus::Online);
                    changed = true;
                }
                if changed {
                    self.writer.save(&existing).await?;
                }
                existing
            }
            Err(RepositoryError::NotFound) => {
                let draft = SensorDraft {
                    id: sensor_id.clone(),
                    status: SensorStatus::Online,
                    coordinate: coord,
                    provenance: Provenance::default(),
                };
                let created = self.writer.save_new(draft).await?;
                self.try_link_to_nearest_tree(&created).await?;
                created
            }
            Err(e) => return Err(e.into()),
        };

        let data = serde_json::to_value(&payload).map_err(|e| {
            ServiceError::Repository(RepositoryError::Internal(format!(
                "failed to serialise mqtt payload: {e}"
            )))
        })?;
        let watermarks = payload.watermarks;

        self.reading_writer
            .record(SensorReadingDraft {
                sensor_id: sensor.id.clone(),
                data,
            })
            .await?;

        self.event_bus
            .publish(DomainEvent::SensorDataReceived {
                sensor_id: sensor.id,
                ts: chrono::Utc::now(),
                watermarks,
            })
            .await;

        Ok(())
    }

    async fn try_link_to_nearest_tree(&self, sensor: &Sensor) -> Result<(), ServiceError> {
        let radius = Distance::new(AUTO_LINK_RADIUS_M).expect("3.0 is a valid distance");
        let Some(mut tree) = self
            .tree_reader
            .find_nearest(sensor.coordinate, radius)
            .await?
        else {
            return Ok(());
        };
        let events = tree.attach_sensor(sensor.id.clone());
        self.tree_writer.save(&tree).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }
}
