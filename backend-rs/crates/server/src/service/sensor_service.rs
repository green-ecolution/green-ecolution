use std::sync::Arc;

use domain::{
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

    /// Ingests one MQTT uplink message: get-or-create the sensor, refresh its
    /// position and status, persist the raw reading, and publish
    /// [`DomainEvent::SensorDataReceived`] for subscribers.
    ///
    /// Idempotent on the sensor record. Auto-link to the nearest tree happens
    /// only on first sight (inside [`Self::ensure_sensor`]).
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %payload.device))]
    pub async fn handle_message(&self, payload: MqttPayload) -> Result<(), ServiceError> {
        let sensor_id = SensorId::new(payload.device.clone())?;
        let coord = Coordinate::new(payload.latitude, payload.longitude)?;
        let mut sensor = self.ensure_sensor(&sensor_id, coord).await?;
        self.update_position(&mut sensor, coord).await?;
        self.record_reading(sensor.id, payload).await
    }

    /// Loads the sensor record by id, or creates a new `Online` record at
    /// `coord` and auto-links it to the nearest tree (within
    /// [`AUTO_LINK_RADIUS_M`]). Position/status of an existing record are
    /// NOT touched here — see [`Self::update_position`].
    async fn ensure_sensor(
        &self,
        sensor_id: &SensorId,
        coord: Coordinate,
    ) -> Result<Sensor, ServiceError> {
        match self.reader.by_id(sensor_id).await {
            Ok(existing) => Ok(existing),
            Err(RepositoryError::NotFound) => {
                let draft = SensorDraft {
                    id: sensor_id.clone(),
                    status: SensorStatus::Online,
                    coordinate: coord,
                    provenance: Provenance::default(),
                };
                let created = self.writer.save_new(draft).await?;
                self.try_link_to_nearest_tree(&created).await?;
                Ok(created)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Bumps the sensor's coordinate and status to `Online` if either drifted
    /// from the incoming uplink. Persists only when something actually
    /// changed; no-op for freshly-created sensors.
    async fn update_position(
        &self,
        sensor: &mut Sensor,
        coord: Coordinate,
    ) -> Result<(), ServiceError> {
        let mut changed = false;
        if sensor.coordinate != coord {
            sensor.move_to(coord);
            changed = true;
        }
        if sensor.status != SensorStatus::Online {
            sensor.change_status(SensorStatus::Online);
            changed = true;
        }
        if changed {
            self.writer.save(sensor).await?;
        }
        Ok(())
    }

    /// Persists the raw payload as a [`SensorReading`] and publishes
    /// [`DomainEvent::SensorDataReceived`] for downstream subscribers (tree
    /// status calibration, dashboards, …).
    async fn record_reading(
        &self,
        sensor_id: SensorId,
        payload: MqttPayload,
    ) -> Result<(), ServiceError> {
        let data = serde_json::to_value(&payload).map_err(|e| {
            ServiceError::Repository(RepositoryError::Internal(format!(
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
            .publish(DomainEvent::SensorDataReceived {
                sensor_id,
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
