use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use domain::{
    Id,
    cluster::{SoilMoistureBucket, SoilMoistureOverview, TreeClusterReader, condition_series},
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    organization::Organization,
    sensor::{
        AcknowledgementNote, DataHealth, DataQualityAcknowledgement, ReadingQualityIssue, Sensor,
        SensorDraft, SensorError, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorView, SensorWriter,
        data::SensorReadingView,
        plausibility::{self, ReadingContext},
        repository::NormalizedValue,
    },
    sensor_model::{SensorAbilityName, SensorModel, SensorModelReader},
    shared::pagination::{Page, Pagination},
    tree::{Tree, TreeReader, TreeWriter, volumetric_thresholds},
};
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;

use super::{OrganizationMismatch, ServiceError, event_bus::EventBus};

pub struct SensorService {
    reader: Arc<dyn SensorReader>,
    writer: Arc<dyn SensorWriter>,
    reading_reader: Arc<dyn SensorReadingReader>,
    reading_writer: Arc<dyn SensorReadingWriter>,
    model_reader: Arc<dyn SensorModelReader>,
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    event_bus: Arc<dyn EventBus>,
}

/// Read model behind `GET /sensors/{id}/data-quality`.
#[derive(Debug)]
pub struct SensorDataQuality {
    pub health: DataHealth,
    pub implausible_recent: i64,
    pub issues: Vec<ReadingQualityIssue>,
    /// Present once someone reviewed the flagged readings. Issues recorded at
    /// or before `at` are the reviewed ones; the frontend splits the list on it.
    pub acknowledged: Option<DataQualityAcknowledgement>,
}

const QUALITY_ISSUE_LIMIT: i64 = 50;

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
        cluster_reader: Arc<dyn TreeClusterReader>,
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
            cluster_reader,
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

    /// Activates a `Prepared` sensor and binds it to `tree_id`. Idempotent
    /// when called with the same `(sensor, tree)` pair after the initial
    /// transition; rejects rebinding to a different tree or activating an
    /// already-active sensor.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id, tree.id = %tree_id))]
    pub async fn activate(
        &self,
        id: &SensorId,
        tree_id: Id<Tree>,
    ) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        let mut tree = self.tree_reader.by_id(tree_id).await?;

        if sensor.organization_id() != tree.organization_id() {
            return Err(OrganizationMismatch::SensorVsTree.into());
        }

        let already_bound_here = tree.sensor_id() == Some(id);
        let activated = sensor.is_activated();

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
        events.extend(sensor.activate(chrono::Utc::now())?);

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

    /// Moves an activated sensor's tree link to `new_tree_id`. Requires the
    /// sensor to be activated; rejects a tree that already holds a different
    /// sensor. Idempotent when the sensor is already linked to `new_tree_id`.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id, tree.id = %new_tree_id))]
    pub async fn reassign_tree(
        &self,
        id: &SensorId,
        new_tree_id: Id<Tree>,
    ) -> Result<SensorView, ServiceError> {
        let sensor = self.reader.by_id(id).await?;
        if !sensor.is_activated() {
            return Err(ServiceError::NotActivated);
        }

        let mut target = self.tree_reader.by_id(new_tree_id).await?;
        if sensor.organization_id() != target.organization_id() {
            return Err(OrganizationMismatch::SensorVsTree.into());
        }
        if target.sensor_id() == Some(id) {
            return Ok(self.reader.view_by_id(id).await?);
        }
        if let Some(other) = target.sensor_id()
            && other != id
        {
            return Err(ServiceError::TreeAlreadyHasSensor);
        }

        let mut events = Vec::new();
        if let Some(mut current) = self.tree_reader.by_sensor_id(id).await? {
            events.extend(current.detach_sensor());
            self.tree_writer.save(&current).await?;
        }
        events.extend(target.attach_sensor(id.clone()));
        self.tree_writer.save(&target).await?;
        self.event_bus.publish_all(events).await;

        Ok(self.reader.view_by_id(id).await?)
    }

    /// Resets an activated sensor back to `Prepared` and removes its tree
    /// link. Idempotent: an already prepared sensor is returned unchanged.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn deactivate(&self, id: &SensorId) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        if !sensor.is_activated() {
            return Ok(self.reader.view_by_id(id).await?);
        }

        let mut events = sensor.deactivate()?;
        if let Some(mut tree) = self.tree_reader.by_sensor_id(id).await? {
            events.extend(tree.detach_sensor());
            self.tree_writer.save(&tree).await?;
        }
        self.writer.save(&sensor).await?;
        self.event_bus.publish_all(events).await;

        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn data_quality(&self, id: &SensorId) -> Result<SensorDataQuality, ServiceError> {
        let view = self.reader.view_by_id(id).await?;
        let sensor = self.reader.by_id(id).await?;
        let issues = self
            .reading_reader
            .quality_issues(id, QUALITY_ISSUE_LIMIT)
            .await?;
        Ok(SensorDataQuality {
            health: view.data_health,
            implausible_recent: view.implausible_recent,
            issues,
            acknowledged: sensor.quality_acknowledged().cloned(),
        })
    }

    /// Records that `by` reviewed this sensor's flagged readings. The
    /// watermark is the server's clock, not a client-supplied timestamp.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn acknowledge_data_quality(
        &self,
        id: &SensorId,
        by: Uuid,
        note: Option<AcknowledgementNote>,
    ) -> Result<SensorDataQuality, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        let events = sensor.acknowledge_data_quality(DataQualityAcknowledgement {
            at: Utc::now(),
            by,
            note,
        });
        if !events.is_empty() {
            self.writer.save(&sensor).await?;
            self.event_bus.publish_all(events).await;
        }
        self.data_quality(id).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %sensor_id))]
    pub async fn view_history(
        &self,
        sensor_id: &SensorId,
        pagination: Pagination,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Page<SensorReadingView>, ServiceError> {
        Ok(self
            .reading_reader
            .view_history(sensor_id, pagination, since, until)
            .await?)
    }

    /// Atomically persists a raw reading + its normalized per-ability values
    /// and publishes [`DomainEvent::SensorDataReceived`] so subscribers can
    /// react without re-parsing the payload. Values are checked against the
    /// domain plausibility rules first; flagged values are stored but removed
    /// from the event payload.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %ingest.sensor_id))]
    pub async fn ingest_reading(&self, mut ingest: ReadingIngest) -> Result<(), ServiceError> {
        let sensor = self.reader.by_id(&ingest.sensor_id).await?;
        let model = self.model_reader.by_id(sensor.model_id()).await?;
        let previous: HashMap<Uuid, (f64, DateTime<Utc>)> = self
            .reading_reader
            .last_plausible_values(&ingest.sensor_id)
            .await?
            .into_iter()
            .filter_map(|p| {
                p.value
                    .to_f64()
                    .map(|v| (p.model_ability_id, (v, p.recorded_at)))
            })
            .collect();

        let recorded_at = Utc::now();
        for value in &mut ingest.normalized {
            let Some(ability) = model.model_ability_by_id(value.model_ability_id) else {
                continue;
            };
            let Some(as_f64) = value.value.to_f64() else {
                continue;
            };
            let ctx = ReadingContext {
                previous: previous.get(&value.model_ability_id).copied(),
                recorded_at,
            };
            value.issue =
                plausibility::evaluate(ability.ability.name, ability.ability.unit, as_f64, &ctx);
        }

        let flagged: HashSet<(SensorAbilityName, i32)> = ingest
            .normalized
            .iter()
            .filter(|v| v.issue.is_some())
            .filter_map(|v| model.model_ability_by_id(v.model_ability_id))
            .map(|a| (a.ability.name, a.depth_cm))
            .collect();

        let before = typed_len(&ingest.typed);
        ingest.typed = match ingest.typed {
            SensorReadings::Watermarks(w) => SensorReadings::Watermarks(
                w.into_iter()
                    .filter(|m| !flagged.contains(&(SensorAbilityName::SoilTension, m.depth)))
                    .collect(),
            ),
            SensorReadings::Volumetrics(v) => SensorReadings::Volumetrics(
                v.into_iter()
                    .filter(|m| !flagged.contains(&(SensorAbilityName::SoilMoisture, m.depth_cm)))
                    .collect(),
            ),
        };
        let after = typed_len(&ingest.typed);

        self.reading_writer
            .record_with_normalized(&ingest.sensor_id, ingest.raw_payload, &ingest.normalized)
            .await?;

        // Only suppress when filtering emptied the payload. An uplink that
        // carried nothing scoreable to begin with keeps its previous behaviour.
        if after == 0 && before > 0 {
            let reasons: Vec<&str> = ingest
                .normalized
                .iter()
                .filter_map(|v| v.issue.map(|i| i.reason().as_str()))
                .collect();
            tracing::warn!(
                sensor.id = %ingest.sensor_id,
                sensor.quality_reasons = ?reasons,
                "no plausible reading in uplink; event suppressed"
            );
            return Ok(());
        }

        self.event_bus
            .publish(DomainEvent::SensorDataReceived(SensorDataReceivedPayload {
                sensor_id: ingest.sensor_id,
                readings: ingest.typed,
            }))
            .await;
        Ok(())
    }

    /// Transfers ownership of an unbound sensor to `target`. A sensor bound
    /// to a tree must transfer via that tree instead (`TreeService::transfer`
    /// cascades it), so this rejects with `SensorBoundToTree` when a tree
    /// link exists.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn transfer(
        &self,
        id: &SensorId,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        if self.tree_reader.by_sensor_id(id).await?.is_some() {
            return Err(ServiceError::SensorBoundToTree);
        }
        let mut sensor = self.reader.by_id(id).await?;
        let events = sensor.transfer_to(target);
        self.writer.save(&sensor).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn soil_moisture_overview(
        &self,
        id: &SensorId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<SoilMoistureOverview, ServiceError> {
        // Distinguish "unknown sensor" (404) from "sensor without readings".
        self.reader.by_id(id).await?;
        let series = self
            .reading_reader
            .soil_moisture_series(id, from, to, bucket)
            .await?;
        let cluster_id = self
            .tree_reader
            .by_sensor_id(id)
            .await?
            .and_then(|tree| tree.cluster_id());
        let (soil, watering_events) = match cluster_id {
            Some(cluster_id) => {
                let view = self.cluster_reader.view_by_id(cluster_id).await?;
                let events = self.cluster_reader.watering_events(cluster_id).await?;
                (view.soil_condition, events)
            }
            None => (None, Vec::new()),
        };
        let thresholds = soil
            .map(|s| {
                series
                    .iter()
                    .filter_map(|d| volumetric_thresholds(s, d.depth_cm))
                    .collect()
            })
            .unwrap_or_default();
        let condition = soil
            .map(|s| condition_series(&series, s))
            .unwrap_or_default();
        Ok(SoilMoistureOverview {
            bucket,
            series,
            thresholds,
            condition,
            watering_events,
        })
    }
}

impl From<SensorError> for ServiceError {
    fn from(err: SensorError) -> Self {
        match err {
            SensorError::AlreadyActivated => ServiceError::AlreadyActivated,
            SensorError::NotActivated => ServiceError::NotActivated,
            SensorError::Validation(e) => ServiceError::Validation(e),
        }
    }
}

fn typed_len(readings: &SensorReadings) -> usize {
    match readings {
        SensorReadings::Watermarks(w) => w.len(),
        SensorReadings::Volumetrics(v) => v.len(),
    }
}
