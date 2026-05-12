use std::sync::Arc;

use domain::{
    Id,
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    sensor::{
        Sensor, SensorDraft, SensorError, SensorId, SensorReader, SensorReadingReader,
        SensorReadingWriter, SensorSearchQuery, SensorStatus, SensorView, SensorWriter,
        data::{MqttPayload, SensorReadingView},
        repository::NormalizedValue,
    },
    sensor_model::{SensorAbilityName, SensorModel, SensorModelReader},
    shared::pagination::{Page, Pagination},
    tree::{Tree, TreeReader, TreeWriter},
};
use rust_decimal::Decimal;

use super::{ServiceError, event_bus::EventBus};

pub struct SensorService {
    reader: Arc<dyn SensorReader>,
    writer: Arc<dyn SensorWriter>,
    reading_reader: Arc<dyn SensorReadingReader>,
    reading_writer: Arc<dyn SensorReadingWriter>,
    model_reader: Arc<dyn SensorModelReader>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    event_bus: Arc<dyn EventBus>,
}

/// Input for [`SensorService::ingest_reading`]. The MQTT parser builds this
/// from the raw bytes plus the looked-up model so the service stays agnostic
/// of any wire format.
#[derive(Debug)]
pub struct ReadingIngest {
    pub sensor_id: SensorId,
    pub raw_payload: serde_json::Value,
    pub normalized: Vec<NormalizedValue>,
    pub typed: SensorReadings,
}

impl SensorService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reader: Arc<dyn SensorReader>,
        writer: Arc<dyn SensorWriter>,
        reading_reader: Arc<dyn SensorReadingReader>,
        reading_writer: Arc<dyn SensorReadingWriter>,
        model_reader: Arc<dyn SensorModelReader>,
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            reading_reader,
            reading_writer,
            model_reader,
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
    pub async fn list_models(&self) -> Result<Vec<SensorModel>, ServiceError> {
        Ok(self.model_reader.list().await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn model_by_id(&self, id: Id<SensorModel>) -> Result<SensorModel, ServiceError> {
        Ok(self.model_reader.by_id(id).await?)
    }

    /// Persists a new sensor. The draft's `model_id` is validated up-front so
    /// callers get a `NotFound` (mapped to 422 at the HTTP layer) rather than
    /// a raw FK violation from the writer.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: SensorDraft) -> Result<SensorView, ServiceError> {
        let _ = self.model_reader.by_id(draft.model_id).await?;
        let sensor = self.writer.save_new(draft).await?;
        Ok(self.reader.view_by_id(&sensor.id).await?)
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

    /// Activates a `Prepared` sensor and binds it to `tree_id`. Idempotent
    /// when called with the same `(sensor, tree)` pair after the initial
    /// transition; rejects rebinding to a different tree or activating an
    /// already-active sensor.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id, tree.id = tree_id))]
    pub async fn activate(&self, id: &SensorId, tree_id: i32) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        let tid = Id::<Tree>::new(tree_id);
        let mut tree = self.tree_reader.by_id(tid).await?;

        let already_bound_here = tree.sensor_id() == Some(id);
        let activated = sensor.status() != SensorStatus::Prepared;

        if already_bound_here && activated {
            return Ok(self.reader.view_by_id(id).await?);
        }
        if let Some(other) = tree.sensor_id()
            && other != id
        {
            return Err(ServiceError::TreeAlreadyHasSensor);
        }
        if activated {
            return Err(ServiceError::AlreadyActivated);
        }

        let mut events = tree.attach_sensor(id.clone());
        events.extend(sensor.activate()?);

        self.tree_writer.save(&tree).await?;
        self.writer.save(&sensor).await?;
        self.event_bus.publish_all(events).await;

        Ok(self.reader.view_by_id(id).await?)
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

    /// Atomically persists a raw reading + its normalized per-ability values
    /// and publishes [`DomainEvent::SensorDataReceived`] so subscribers can
    /// react without re-parsing the payload.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %ingest.sensor_id))]
    pub async fn ingest_reading(&self, ingest: ReadingIngest) -> Result<(), ServiceError> {
        self.reading_writer
            .record_with_normalized(&ingest.sensor_id, ingest.raw_payload, &ingest.normalized)
            .await?;
        self.event_bus
            .publish(DomainEvent::SensorDataReceived(SensorDataReceivedPayload {
                sensor_id: ingest.sensor_id,
                readings: ingest.typed,
            }))
            .await;
        Ok(())
    }

    /// Ingests one legacy EcoDrizzler MQTT uplink: looks up the sensor's
    /// model so it can resolve the normalized ability ids, then delegates to
    /// [`Self::ingest_reading`]. The sensor must already exist (registration
    /// is now an explicit step).
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %payload.device))]
    pub async fn handle_message(&self, payload: MqttPayload) -> Result<(), ServiceError> {
        let sensor_id = SensorId::new(payload.device.clone())?;
        let mut sensor = self.reader.by_id(&sensor_id).await?;
        self.bump_online(&mut sensor).await?;

        let model = self.model_reader.by_id(sensor.model_id()).await?;
        let raw_payload = serde_json::to_value(&payload).map_err(|e| {
            ServiceError::Repository(domain::RepositoryError::Internal(format!(
                "failed to serialise mqtt payload: {e}"
            )))
        })?;
        let normalized = normalize_eco_drizzler(&model, &payload);
        let watermarks = payload.watermarks;

        self.ingest_reading(ReadingIngest {
            sensor_id,
            raw_payload,
            normalized,
            typed: SensorReadings::Watermarks(watermarks),
        })
        .await
    }

    async fn bump_online(&self, sensor: &mut Sensor) -> Result<(), ServiceError> {
        if sensor.status() != SensorStatus::Online {
            sensor.change_status(SensorStatus::Online);
            self.writer.save(sensor).await?;
        }
        Ok(())
    }
}

fn normalize_eco_drizzler(model: &SensorModel, payload: &MqttPayload) -> Vec<NormalizedValue> {
    let mut out = Vec::with_capacity(payload.watermarks.len() + 2);
    for w in &payload.watermarks {
        if let Some(model_ability_id) =
            model.ability_id_for(SensorAbilityName::SoilTension, w.depth)
        {
            out.push(NormalizedValue {
                model_ability_id,
                value: Decimal::from(w.centibar),
            });
        }
    }
    if let Some(model_ability_id) = model.ability_id_for(SensorAbilityName::Temperature, 15) {
        out.push(NormalizedValue {
            model_ability_id,
            value: Decimal::from_f64_retain(payload.temperature).unwrap_or_default(),
        });
    }
    if let Some(model_ability_id) = model.ability_id_for(SensorAbilityName::Humidity, 15) {
        out.push(NormalizedValue {
            model_ability_id,
            value: Decimal::from_f64_retain(payload.humidity).unwrap_or_default(),
        });
    }
    out
}

impl From<SensorError> for ServiceError {
    fn from(err: SensorError) -> Self {
        match err {
            SensorError::AlreadyActivated => ServiceError::AlreadyActivated,
            SensorError::Validation(e) => ServiceError::InvalidInput(e.to_string()),
        }
    }
}
