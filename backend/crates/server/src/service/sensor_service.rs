use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use domain::{
    Id, RepositoryError,
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    module::ModuleActivationReader,
    organization::Organization,
    sensor::{
        AcknowledgementNote, DataHealth, DataQualityAcknowledgement, ReadingQualityIssue, Sensor,
        SensorDraft, SensorError, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorView, SensorWriter,
        data::{SensorReadingView, SoilMoistureBucket, SoilMoistureDepthSeries},
        plausibility::{self, ReadingContext},
        repository::NormalizedValue,
    },
    sensor_model::{SensorAbilityName, SensorModel, SensorModelReader},
    shared::pagination::{Page, Pagination, SearchPage},
    target::{TargetPurpose, TargetRef},
};
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;

use super::{
    OrganizationMismatch, ServiceError,
    event_bus::EventBus,
    targets::{TargetRegistry, ensure_module_active},
};

pub struct SensorService {
    reader: Arc<dyn SensorReader>,
    writer: Arc<dyn SensorWriter>,
    reading_reader: Arc<dyn SensorReadingReader>,
    reading_writer: Arc<dyn SensorReadingWriter>,
    model_reader: Arc<dyn SensorModelReader>,
    targets: Arc<TargetRegistry>,
    activation: Arc<dyn ModuleActivationReader>,
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
        targets: Arc<TargetRegistry>,
        activation: Arc<dyn ModuleActivationReader>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            reader,
            writer,
            reading_reader,
            reading_writer,
            model_reader,
            targets,
            activation,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: SensorSearchQuery,
        pagination: Pagination,
    ) -> Result<SearchPage<SensorView>, ServiceError> {
        let mut page = self.reader.view_search(query, pagination).await?;
        self.enrich(&mut page.page.items).await?;
        Ok(page)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, ServiceError> {
        let mut view = self.reader.view_by_id(id).await?;
        self.enrich(std::slice::from_mut(&mut view)).await?;
        Ok(view)
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
        let mut views = self.reader.view_by_ids(ids).await?;
        self.enrich(&mut views).await?;
        Ok(views)
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
        self.view_by_id(&sensor.id).await
    }

    /// The target must exist, take sensors, belong to a module active in its
    /// organization and share the sensor's organization.
    async fn admit_for(&self, sensor: &Sensor, target: &TargetRef) -> Result<(), ServiceError> {
        let admitted = self.targets.admit(target, TargetPurpose::Sensor).await?;
        ensure_module_active(
            self.activation.as_ref(),
            &target.module,
            admitted.organization_id,
        )
        .await?;
        if admitted.organization_id != sensor.organization_id() {
            return Err(OrganizationMismatch::SensorVsTarget.into());
        }
        Ok(())
    }

    /// Activates a `Prepared` sensor and attaches it to `target`. Idempotent
    /// for the same pair; rejects an already active sensor and a target that
    /// carries another sensor.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn activate(
        &self,
        id: &SensorId,
        target: TargetRef,
    ) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        if sensor.activated_at().is_some() && sensor.target() == Some(&target) {
            return self.view_by_id(id).await;
        }
        self.admit_for(&sensor, &target).await?;
        let events = sensor.activate(target, Utc::now())?;
        self.save_attachment(&sensor).await?;
        self.event_bus.publish_all(events).await;
        self.view_by_id(id).await
    }

    /// Attaches without activating, as the tree form does with a prepared
    /// sensor.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn attach(&self, id: &SensorId, target: TargetRef) -> Result<(), ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        if sensor.target() == Some(&target) {
            return Ok(());
        }
        self.admit_for(&sensor, &target).await?;
        let events = sensor.attach(target)?;
        self.save_attachment(&sensor).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    /// Moves an activated sensor to `target`. Idempotent when it is already
    /// there.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn retarget(
        &self,
        id: &SensorId,
        target: TargetRef,
    ) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        if !sensor.is_activated() {
            return Err(ServiceError::NotActivated);
        }
        if sensor.target() == Some(&target) {
            return self.view_by_id(id).await;
        }
        self.admit_for(&sensor, &target).await?;
        let events = sensor.retarget(target)?;
        self.save_attachment(&sensor).await?;
        self.event_bus.publish_all(events).await;
        self.view_by_id(id).await
    }

    /// Resets an activated sensor back to `Prepared` and releases its target.
    /// Idempotent: an already prepared sensor is returned unchanged.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn deactivate(&self, id: &SensorId) -> Result<SensorView, ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        if !sensor.is_activated() {
            return self.view_by_id(id).await;
        }
        let events = sensor.deactivate()?;
        self.save_attachment(&sensor).await?;
        self.event_bus.publish_all(events).await;
        self.view_by_id(id).await
    }

    /// Frees the sensor of a target that is going away. The sensor stays active.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn release_target(&self, target: &TargetRef) -> Result<(), ServiceError> {
        let Some(mut sensor) = self.reader.by_target(target).await? else {
            return Ok(());
        };
        let events = sensor.detach();
        self.writer.save(&sensor).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    async fn save_attachment(&self, sensor: &Sensor) -> Result<(), ServiceError> {
        self.writer.save(sensor).await.map_err(|e| match e {
            RepositoryError::AlreadyExists(_) => ServiceError::TargetAlreadyHasSensor,
            e => e.into(),
        })
    }

    async fn enrich(&self, views: &mut [SensorView]) -> Result<(), ServiceError> {
        let targets: Vec<TargetRef> = views.iter().filter_map(|v| v.target.clone()).collect();
        if targets.is_empty() {
            return Ok(());
        }
        let locations = self.targets.locate_all(&targets).await?;
        let labels = self.targets.label_all(&targets).await?;
        for view in views.iter_mut() {
            if let Some(target) = &view.target {
                view.location = locations.get(target).copied();
                view.target_label = labels.get(target).cloned();
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn delete(&self, id: &SensorId) -> Result<(), ServiceError> {
        let mut sensor = self.reader.by_id(id).await?;
        let events = sensor.detach();
        self.writer.delete(id).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
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
                target: sensor.target().cloned(),
                readings: ingest.typed,
            }))
            .await;
        Ok(())
    }

    /// Transfers ownership of an unattached sensor to `target`. An attached
    /// sensor moves with its target instead (the module that owns the target
    /// cascades the transfer), so this rejects with `SensorBoundToTarget`.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn transfer(
        &self,
        id: &SensorId,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let sensor = self.reader.by_id(id).await?;
        if sensor.target().is_some() {
            return Err(ServiceError::SensorBoundToTarget);
        }
        self.save_transfer(sensor, target).await
    }

    /// Skips the bound check of [`Self::transfer`]: the module calls it while moving the target itself.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn transfer_with_target(
        &self,
        id: &SensorId,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let sensor = self.reader.by_id(id).await?;
        self.save_transfer(sensor, target).await
    }

    async fn save_transfer(
        &self,
        mut sensor: Sensor,
        target: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let events = sensor.transfer_to(target);
        self.writer.save(&sensor).await?;
        self.event_bus.publish_all(events).await;
        Ok(())
    }

    /// Bucketed volumetric readings per probe depth. An unknown sensor is a
    /// 404, a known one without readings an empty list.
    #[tracing::instrument(level = "debug", skip_all, fields(sensor.id = %id))]
    pub async fn soil_moisture_series(
        &self,
        id: &SensorId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<Vec<SoilMoistureDepthSeries>, ServiceError> {
        self.reader.by_id(id).await?;
        Ok(self
            .reading_reader
            .soil_moisture_series(id, from, to, bucket)
            .await?)
    }
}

impl From<SensorError> for ServiceError {
    fn from(err: SensorError) -> Self {
        match err {
            SensorError::AlreadyActivated => ServiceError::AlreadyActivated,
            SensorError::NotActivated => ServiceError::NotActivated,
            SensorError::AlreadyAttached => ServiceError::SensorAlreadyAttached,
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
