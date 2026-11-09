//! One export pass per organization: the modules' current objects are diffed
//! against the refs of the last pass, and only the differences are written.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use domain::{
    Id, RepositoryError,
    clock::Clock,
    data_export::{
        ExportEntity, ExportLease, ExportProperty, ExportReading, ExportReadingSource, ExportRef,
        ExportRefReader, ExportRefWriter, ExportRunStats, ExportRunStore, ExportState,
        ExportTargetConfig, ExportThing, ObservationRow, RawStream, RemoteId, RunResult,
        SENSOR_KEY, SensorThingsSink, SinkError, SinkSession, StateStream, device_sensor_key,
        export_scope, ge_key, location_key, property_key, raw_property, raw_stream_key,
        state_stream_key, v7_floor,
    },
    module::{ModuleActivationReader, ModuleKey},
    organization::{Organization, OrganizationReader},
};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::export_registry::ExportRegistry;

/// Headroom between the run budget and the lease, so a pass that stops at its
/// budget still finishes inside its lease.
const LEASE_MARGIN_SECS: i64 = 60;

pub struct SyncSettings {
    pub interval_secs: i64,
    pub run_budget: Duration,
    /// Readings per batch; values below 1 are read as 1.
    pub batch_size: i64,
    /// Readings younger than this wait for a later pass, so one that commits after
    /// a higher id is not passed by the cursor.
    pub reading_settle: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Sink(#[from] SinkError),
    #[error("the export pass lost its lease to another replica")]
    LeaseLost,
}

/// Shown to everyone with `setting:read`, so it never carries driver detail.
const INTERNAL_ERROR_TEXT: &str = "internal error";

impl SyncError {
    fn status_text(&self) -> String {
        match self {
            Self::Repository(_) => INTERNAL_ERROR_TEXT.to_string(),
            Self::Sink(e) => e.to_string(),
            Self::LeaseLost => self.to_string(),
        }
    }
}

pub struct DataExportSync {
    runs: Arc<dyn ExportRunStore>,
    refs: Arc<dyn ExportRefReader>,
    ref_writer: Arc<dyn ExportRefWriter>,
    readings: Arc<dyn ExportReadingSource>,
    sink: Arc<dyn SensorThingsSink>,
    registry: Arc<ExportRegistry>,
    activation: Arc<dyn ModuleActivationReader>,
    organizations: Arc<dyn OrganizationReader>,
    clock: Arc<dyn Clock>,
    settings: SyncSettings,
}

impl DataExportSync {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runs: Arc<dyn ExportRunStore>,
        refs: Arc<dyn ExportRefReader>,
        ref_writer: Arc<dyn ExportRefWriter>,
        readings: Arc<dyn ExportReadingSource>,
        sink: Arc<dyn SensorThingsSink>,
        registry: Arc<ExportRegistry>,
        activation: Arc<dyn ModuleActivationReader>,
        organizations: Arc<dyn OrganizationReader>,
        clock: Arc<dyn Clock>,
        settings: SyncSettings,
    ) -> Self {
        Self {
            runs,
            refs,
            ref_writer,
            readings,
            sink,
            registry,
            activation,
            organizations,
            clock,
            settings: SyncSettings {
                batch_size: settings.batch_size.max(1),
                ..settings
            },
        }
    }

    pub async fn run_due(&self) {
        let due = match self.runs.due(self.settings.interval_secs).await {
            Ok(due) => due,
            Err(e) => {
                tracing::warn!(error = %e, "could not list the due export targets");
                return;
            }
        };
        for org in due {
            if let Err(e) = self.sync(org).await {
                tracing::warn!(organization.id = %org, error = %e, "export pass failed");
            }
        }
    }

    /// `None` when the target is not leased: another replica runs it or it is not due.
    #[tracing::instrument(level = "info", skip_all, fields(organization.id = %org))]
    pub async fn sync(&self, org: Id<Organization>) -> Result<Option<ExportRunStats>, SyncError> {
        let token = Uuid::now_v7();
        let Some(leased) = self
            .runs
            .try_lease(org, token, self.lease_secs(), self.settings.interval_secs)
            .await?
        else {
            return Ok(None);
        };
        let lease = ExportLease {
            generation: leased.target.activated_at,
            token,
        };
        let outcome = self.pass(org, lease, &leased.target.config).await;
        let result = match &outcome {
            Ok(stats) => RunResult::Succeeded(*stats),
            Err(e) => {
                if let SyncError::Repository(detail) = e {
                    tracing::error!(organization.id = %org, error = %detail, "export pass failed on the database");
                }
                RunResult::Failed {
                    error: e.status_text(),
                    base_interval_secs: self.settings.interval_secs,
                }
            }
        };
        let finished = self.runs.finish(org, &lease, &result).await;
        match (outcome, finished) {
            (Ok(stats), finished) => {
                finished?;
                Ok(Some(stats))
            }
            (Err(pass_error), Err(finish_error)) => {
                tracing::warn!(organization.id = %org, error = %finish_error, "recording the failed export pass failed");
                Err(pass_error)
            }
            (Err(pass_error), Ok(())) => Err(pass_error),
        }
    }

    fn lease_secs(&self) -> i64 {
        i64::try_from(self.settings.run_budget.as_secs())
            .unwrap_or(i64::MAX)
            .saturating_add(LEASE_MARGIN_SECS)
    }

    async fn pass(
        &self,
        org: Id<Organization>,
        lease: ExportLease,
        config: &ExportTargetConfig,
    ) -> Result<ExportRunStats, SyncError> {
        let started = self.clock.now();
        let hierarchy = self.organizations.hierarchy().await?;
        let targets = self.runs.target_orgs().await?;
        let scope = export_scope(&hierarchy, &targets, org);
        let session = self.sink.open(config).await?;
        let mut pass = Pass {
            org,
            lease,
            lease_secs: self.lease_secs(),
            renewed_at: started,
            session,
            refs: self.refs.refs(org).await?,
            writer: self.ref_writer.as_ref(),
            runs: self.runs.as_ref(),
            readings: self.readings.as_ref(),
            stats: ExportRunStats::default(),
            clock: self.clock.as_ref(),
            started,
            budget: self.settings.run_budget,
            refused: HashSet::new(),
        };
        let sensor = pass.sensor().await?;
        let mut raw = RawScope::default();
        for (module, catalog) in self.registry.catalogs() {
            if pass.out_of_budget() {
                break;
            }
            let active = self.active_in(&module, &scope).await?;
            // A module off in the whole scope is not asked at all, so its objects
            // are not mistaken for deleted ones.
            if active.is_empty() {
                continue;
            }
            let things = catalog.snapshot(&active).await?;
            raw.add(&module, &active, &things);
            pass.module(&module, &things, &sensor).await?;
        }
        if !pass.out_of_budget() {
            pass.raw_phase(&raw, self.settings.batch_size, self.settings.reading_settle)
                .await?;
        }
        Ok(pass.stats)
    }

    async fn active_in(
        &self,
        module: &ModuleKey,
        scope: &BTreeSet<Id<Organization>>,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        let active = self.activation.active_orgs(module).await?;
        Ok(active.intersection(scope).copied().collect())
    }
}

/// What one pass carries along: the session, the refs cache and the counters.
struct Pass<'a> {
    org: Id<Organization>,
    /// The stores drop every write of a pass that no longer holds this lease.
    lease: ExportLease,
    lease_secs: i64,
    renewed_at: DateTime<Utc>,
    session: Box<dyn SinkSession>,
    refs: HashMap<String, ExportRef>,
    writer: &'a dyn ExportRefWriter,
    runs: &'a dyn ExportRunStore,
    readings: &'a dyn ExportReadingSource,
    stats: ExportRunStats,
    clock: &'a dyn Clock,
    started: DateTime<Utc>,
    budget: Duration,
    /// Remote entities the target rejected in this pass, so later readings of the
    /// same stream do not ask again.
    refused: HashSet<String>,
}

/// What became of one reading in a batch.
enum RawRow {
    Stream(String, RemoteId),
    Skipped,
    /// The budget ran out before a remote entity the reading needs was created.
    OutOfBudget,
}

enum BatchEnd {
    Complete,
    /// Stopped at the budget; readings up to `done_until` are handled.
    Cut {
        done_until: Option<Uuid>,
    },
}

struct SnapshotThing {
    name: String,
    located: bool,
}

/// What the things phase saw, so readings go only to things exported in this pass.
#[derive(Default)]
struct RawScope {
    active: Vec<(ModuleKey, Id<Organization>)>,
    things: HashMap<String, SnapshotThing>,
}

impl RawScope {
    fn add(
        &mut self,
        module: &ModuleKey,
        active: &BTreeSet<Id<Organization>>,
        things: &[ExportThing],
    ) {
        self.active
            .extend(active.iter().map(|org| (module.clone(), *org)));
        self.things.extend(things.iter().map(|t| {
            let seen = SnapshotThing {
                name: t.name.clone(),
                located: t.coordinate.is_some(),
            };
            (t.key(), seen)
        }));
    }
}

fn observation_row(reading: &ExportReading) -> ObservationRow {
    let mut parameters = Map::new();
    parameters.insert(
        "ge_reading_id".into(),
        Value::String(reading.reading_id.to_string()),
    );
    if let Some(issue) = &reading.quality_issue {
        parameters.insert("quality_issue".into(), Value::String(issue.clone()));
    }
    ObservationRow {
        phenomenon_time: reading.measured_at,
        result: reading.value,
        parameters,
    }
}

fn plain_ref(ge_key: impl Into<String>, entity: ExportEntity, remote_id: RemoteId) -> ExportRef {
    ExportRef {
        ge_key: ge_key.into(),
        entity,
        remote_id,
        content_hash: None,
        last_value: None,
        deleted_marked: false,
    }
}

impl Pass<'_> {
    fn out_of_budget(&self) -> bool {
        let elapsed = (self.clock.now() - self.started)
            .to_std()
            .unwrap_or_default();
        elapsed >= self.budget
    }

    /// A single slow step can outlast the margin between budget and lease, so the
    /// lease is extended while the pass works. A third of the lease leaves room
    /// for a step of several requests between two renewals.
    async fn keep_lease(&mut self) -> Result<(), SyncError> {
        let now = self.clock.now();
        if (now - self.renewed_at).num_seconds() < self.lease_secs / 3 {
            return Ok(());
        }
        if !self
            .runs
            .renew_lease(self.org, &self.lease, self.lease_secs)
            .await?
        {
            return Err(SyncError::LeaseLost);
        }
        self.renewed_at = now;
        Ok(())
    }

    async fn remember(&mut self, export_ref: ExportRef) -> Result<(), RepositoryError> {
        self.writer
            .put_ref(self.org, &self.lease, &export_ref)
            .await?;
        self.refs.insert(export_ref.ge_key.clone(), export_ref);
        Ok(())
    }

    /// A rejected entity is skipped and the pass goes on; every other sink error ends it.
    fn tolerate<T>(
        &mut self,
        key: &str,
        result: Result<T, SinkError>,
    ) -> Result<Option<T>, SinkError> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.aborts_pass() => Err(e),
            Err(e) => {
                let status = match e {
                    SinkError::Rejected { status, .. } => Some(status),
                    _ => None,
                };
                tracing::warn!(
                    ge_key = key,
                    status,
                    "export target rejected an entity; skipped"
                );
                self.stats.skipped += 1;
                Ok(None)
            }
        }
    }

    /// Every datastream needs the sensor, so a sensor the target refuses ends the pass.
    async fn sensor(&mut self) -> Result<RemoteId, SyncError> {
        if let Some(existing) = self.refs.get(SENSOR_KEY) {
            return Ok(existing.remote_id.clone());
        }
        let id = self.session.ensure_sensor().await?;
        self.remember(plain_ref(SENSOR_KEY, ExportEntity::Sensor, id.clone()))
            .await?;
        Ok(id)
    }

    async fn module(
        &mut self,
        module: &ModuleKey,
        things: &[ExportThing],
        sensor: &RemoteId,
    ) -> Result<(), SyncError> {
        for thing in things {
            if self.out_of_budget() {
                return Ok(());
            }
            self.keep_lease().await?;
            self.thing(thing, sensor).await?;
        }
        let seen: HashSet<String> = things.iter().map(ExportThing::key).collect();
        self.mark_vanished(module, &seen).await
    }

    async fn thing(&mut self, thing: &ExportThing, sensor: &RemoteId) -> Result<(), SyncError> {
        let key = thing.key();
        let Some(id) = self.upsert(&key, thing).await? else {
            return Ok(());
        };
        self.locate(&key, &id, thing).await?;
        // FROST derives an Observation's FeatureOfInterest from the Thing's Location
        // and rejects every Observation of a Thing without one.
        if thing.coordinate.is_none() {
            return Ok(());
        }
        for state in &thing.states {
            self.state(&key, &id, thing, state, sensor).await?;
        }
        Ok(())
    }

    async fn upsert(
        &mut self,
        key: &str,
        thing: &ExportThing,
    ) -> Result<Option<RemoteId>, SyncError> {
        let content = thing.content_fingerprint();
        let Some(existing) = self.refs.get(key).cloned() else {
            return self.create(key, thing, content).await;
        };
        if existing.content_hash.as_deref() == Some(content.as_str()) && !existing.deleted_marked {
            return Ok(Some(existing.remote_id));
        }
        let updated = self.session.update_thing(&existing.remote_id, thing).await;
        if self.tolerate(key, updated)?.is_some() {
            let id = existing.remote_id.clone();
            self.remember(ExportRef {
                content_hash: Some(content),
                deleted_marked: false,
                ..existing
            })
            .await?;
            self.stats.things_written += 1;
            return Ok(Some(id));
        }
        Ok(Some(existing.remote_id))
    }

    async fn create(
        &mut self,
        key: &str,
        thing: &ExportThing,
        content: String,
    ) -> Result<Option<RemoteId>, SyncError> {
        let created = self.session.create_thing(thing).await;
        let Some((id, new)) = self.tolerate(key, created)? else {
            return Ok(None);
        };
        self.remember(ExportRef {
            content_hash: Some(content),
            ..plain_ref(key, ExportEntity::Thing, id.clone())
        })
        .await?;
        // An adopted Thing keeps whatever Location it had; no fingerprint forces a move.
        let location = if new {
            thing.location_fingerprint()
        } else {
            None
        };
        self.remember(ExportRef {
            content_hash: location,
            ..plain_ref(location_key(key), ExportEntity::Location, id.clone())
        })
        .await?;
        self.stats.things_written += 1;
        Ok(Some(id))
    }

    async fn locate(
        &mut self,
        key: &str,
        id: &RemoteId,
        thing: &ExportThing,
    ) -> Result<(), SyncError> {
        let Some(fingerprint) = thing.location_fingerprint() else {
            return Ok(());
        };
        let location = location_key(key);
        let stored = self
            .refs
            .get(&location)
            .and_then(|r| r.content_hash.as_deref());
        if stored == Some(fingerprint.as_str()) {
            return Ok(());
        }
        let moved = self.session.move_thing(id, thing).await;
        if self.tolerate(&location, moved)?.is_some() {
            self.remember(ExportRef {
                content_hash: Some(fingerprint),
                ..plain_ref(location, ExportEntity::Location, id.clone())
            })
            .await?;
        }
        Ok(())
    }

    async fn state(
        &mut self,
        thing_key: &str,
        thing_id: &RemoteId,
        thing: &ExportThing,
        state: &ExportState,
        sensor: &RemoteId,
    ) -> Result<(), SyncError> {
        let Some(observed_property) = self.observed_property(&state.property).await? else {
            return Ok(());
        };
        let stream_key = state_stream_key(&state.property, thing_key);
        let stream = StateStream {
            ge_key: &stream_key,
            thing: thing_id,
            thing_name: &thing.name,
            property: &state.property,
            sensor,
            observed_property: &observed_property,
        };
        let Some(datastream) = self.datastream(&stream).await? else {
            return Ok(());
        };
        let value = state.value.to_json();
        if datastream.last_value.as_ref() == Some(&value) {
            return Ok(());
        }
        let posted = self
            .session
            .post_observation(&datastream.remote_id, &state.value, state.observed_at)
            .await;
        if self.tolerate(&stream_key, posted)?.is_some() {
            self.remember(ExportRef {
                last_value: Some(value),
                ..datastream
            })
            .await?;
            self.stats.observations_written += 1;
        }
        Ok(())
    }

    async fn observed_property(
        &mut self,
        property: &ExportProperty,
    ) -> Result<Option<RemoteId>, SyncError> {
        let key = property_key(property);
        if let Some(existing) = self.refs.get(&key) {
            return Ok(Some(existing.remote_id.clone()));
        }
        let ensured = self.session.ensure_observed_property(property).await;
        let Some(id) = self.tolerate(&key, ensured)? else {
            return Ok(None);
        };
        self.remember(plain_ref(key, ExportEntity::ObservedProperty, id.clone()))
            .await?;
        Ok(Some(id))
    }

    async fn datastream(
        &mut self,
        stream: &StateStream<'_>,
    ) -> Result<Option<ExportRef>, SyncError> {
        if let Some(existing) = self.refs.get(stream.ge_key) {
            return Ok(Some(existing.clone()));
        }
        let ensured = self.session.ensure_state_datastream(stream).await;
        let Some(id) = self.tolerate(stream.ge_key, ensured)? else {
            return Ok(None);
        };
        let created = plain_ref(stream.ge_key, ExportEntity::Datastream, id);
        self.remember(created.clone()).await?;
        Ok(Some(created))
    }

    async fn mark_vanished(
        &mut self,
        module: &ModuleKey,
        seen: &HashSet<String>,
    ) -> Result<(), SyncError> {
        let prefix = format!("{}:", module.as_str());
        let mut vanished: Vec<ExportRef> = self
            .refs
            .values()
            .filter(|r| r.entity == ExportEntity::Thing && r.ge_key.starts_with(&prefix))
            .filter(|r| !r.deleted_marked && !seen.contains(&r.ge_key))
            .cloned()
            .collect();
        vanished.sort_by(|a, b| a.ge_key.cmp(&b.ge_key));
        for thing in vanished {
            if self.out_of_budget() {
                return Ok(());
            }
            self.keep_lease().await?;
            let marked = self
                .session
                .mark_thing_deleted(&thing.remote_id, self.clock.now())
                .await;
            if self.tolerate(&thing.ge_key, marked)?.is_some() {
                self.remember(ExportRef {
                    deleted_marked: true,
                    ..thing
                })
                .await?;
            }
        }
        Ok(())
    }

    /// The cursor moves only past readings that were posted, so a pass that ends
    /// early resends at most the last batch.
    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %self.org))]
    async fn raw_phase(
        &mut self,
        scope: &RawScope,
        batch_size: i64,
        settle: Duration,
    ) -> Result<(), SyncError> {
        let mut cursor = match self.runs.cursor(self.org).await? {
            Some(cursor) => cursor,
            None => v7_floor(self.lease.generation),
        };
        let settled = chrono::TimeDelta::from_std(settle)
            .ok()
            .and_then(|settle| self.clock.now().checked_sub_signed(settle))
            .unwrap_or(DateTime::<Utc>::MIN_UTC);
        let before = v7_floor(settled);
        while !self.out_of_budget() {
            self.keep_lease().await?;
            let batch = self
                .readings
                .readings_after(&scope.active, cursor, before, batch_size)
                .await?;
            let Some(until) = batch.scanned_until.filter(|_| batch.scanned > 0) else {
                return Ok(());
            };
            if let BatchEnd::Cut { done_until } = self.raw_batch(&batch.rows, scope).await? {
                if let Some(done) = done_until {
                    self.runs
                        .advance_cursor(self.org, &self.lease, done)
                        .await?;
                }
                return Ok(());
            }
            self.runs
                .advance_cursor(self.org, &self.lease, until)
                .await?;
            cursor = until;
            if batch.scanned < batch_size {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Rows arrive ordered by reading, so a cut leaves every earlier reading
    /// complete; the rows of the reading it stopped at wait for the next pass.
    async fn raw_batch(
        &mut self,
        rows: &[ExportReading],
        scope: &RawScope,
    ) -> Result<BatchEnd, SyncError> {
        let mut routed = Vec::new();
        let mut cut_at = None;
        for row in rows {
            match self.raw_row(row, scope).await? {
                RawRow::Stream(key, datastream) => routed.push((row, key, datastream)),
                RawRow::Skipped => {}
                RawRow::OutOfBudget => {
                    cut_at = Some(row.reading_id);
                    break;
                }
            }
        }
        let mut streams: BTreeMap<String, (RemoteId, Vec<ObservationRow>)> = BTreeMap::new();
        for (row, key, datastream) in routed {
            if cut_at.is_some_and(|cut| row.reading_id >= cut) {
                continue;
            }
            streams
                .entry(key)
                .or_insert_with(|| (datastream, Vec::new()))
                .1
                .push(observation_row(row));
        }
        if !streams.is_empty() {
            self.keep_lease().await?;
            let batches: Vec<_> = streams.into_values().collect();
            let outcome = self.session.post_observation_batches(&batches).await?;
            self.stats.observations_written += outcome.accepted;
            self.stats.skipped += outcome.rejected;
        }
        Ok(match cut_at {
            None => BatchEnd::Complete,
            Some(cut) => BatchEnd::Cut {
                done_until: rows
                    .iter()
                    .map(|r| r.reading_id)
                    .filter(|id| *id < cut)
                    .max(),
            },
        })
    }

    /// The datastream a reading goes to. Skipped (and counted) when its Thing is
    /// not exported in this pass or a remote entity was rejected.
    async fn raw_row(
        &mut self,
        row: &ExportReading,
        scope: &RawScope,
    ) -> Result<RawRow, SyncError> {
        let thing_key = ge_key(&row.target);
        let thing_id = self
            .refs
            .get(&thing_key)
            .filter(|r| r.entity == ExportEntity::Thing)
            .map(|r| r.remote_id.clone());
        let exported = scope.things.get(&thing_key).filter(|t| t.located);
        let Some((thing, thing_id)) = exported.zip(thing_id) else {
            self.stats.skipped += 1;
            return Ok(RawRow::Skipped);
        };
        // serde_json would send NaN or infinity as a null result.
        if !row.value.is_finite() {
            self.stats.skipped += 1;
            return Ok(RawRow::Skipped);
        }
        let key = raw_stream_key(row);
        if let Some(existing) = self.refs.get(&key) {
            return Ok(RawRow::Stream(key, existing.remote_id.clone()));
        }
        if self.refused.contains(&key) {
            self.stats.skipped += 1;
            return Ok(RawRow::Skipped);
        }
        // Creating a stream costs several requests; check the budget and the lease
        // before each one rather than once per batch.
        if self.out_of_budget() {
            return Ok(RawRow::OutOfBudget);
        }
        self.keep_lease().await?;
        let Some(id) = self
            .raw_datastream(row, &key, &thing.name, &thing_id)
            .await?
        else {
            self.refused.insert(key);
            return Ok(RawRow::Skipped);
        };
        Ok(RawRow::Stream(key, id))
    }

    async fn raw_datastream(
        &mut self,
        row: &ExportReading,
        key: &str,
        thing_name: &str,
        thing: &RemoteId,
    ) -> Result<Option<RemoteId>, SyncError> {
        let Some(sensor) = self.device_sensor(row).await? else {
            return Ok(None);
        };
        let property = raw_property(row.ability, row.unit);
        let Some(observed_property) = self.observed_property(&property).await? else {
            return Ok(None);
        };
        // Depth 0 means a sensor without depth (battery, distance).
        let name = match row.depth_cm {
            0 => format!("{thing_name} · {}", property.label),
            depth => format!("{thing_name} · {} {depth} cm", property.label),
        };
        let stream = RawStream {
            ge_key: key,
            name: &name,
            thing,
            property: &property,
            sensor: &sensor,
            observed_property: &observed_property,
        };
        let ensured = self.session.ensure_raw_datastream(&stream).await;
        let Some(id) = self.tolerate(key, ensured)? else {
            return Ok(None);
        };
        self.remember(plain_ref(key, ExportEntity::Datastream, id.clone()))
            .await?;
        Ok(Some(id))
    }

    async fn device_sensor(&mut self, row: &ExportReading) -> Result<Option<RemoteId>, SyncError> {
        let key = device_sensor_key(&row.sensor_id);
        if let Some(existing) = self.refs.get(&key) {
            return Ok(Some(existing.remote_id.clone()));
        }
        if self.refused.contains(&key) {
            self.stats.skipped += 1;
            return Ok(None);
        }
        let ensured = self
            .session
            .ensure_device_sensor(&row.sensor_id, &row.sensor_model)
            .await;
        let Some(id) = self.tolerate(&key, ensured)? else {
            self.refused.insert(key);
            return Ok(None);
        };
        self.remember(plain_ref(key, ExportEntity::Sensor, id.clone()))
            .await?;
        Ok(Some(id))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet, HashMap},
        sync::{Arc, Mutex},
    };

    use chrono::{DateTime, TimeZone, Utc};
    use domain::{
        Id, RepositoryError,
        authorization::OrgHierarchy,
        data_export::*,
        module::{ActivationResolution, ModuleActivationReader, ModuleKey, OwnSwitch},
        organization::{Organization, OrganizationReader, OrganizationResourceCounts},
        sensor_model::{SensorAbilityName, SensorAbilityUnit},
        shared::coordinates::Coordinate,
        target::TargetRef,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    const WATERING_STATUS: ExportProperty = ExportProperty {
        key: "watering_status",
        label: "Bewässerungsstatus",
        description: "Derived watering status",
        kind: ObservationKind::Category,
        unit: None,
    };

    fn org(n: u128) -> Id<Organization> {
        Id::new(Uuid::from_u128(n))
    }

    fn generation() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 1, 12, 0, 0).unwrap()
    }

    #[derive(Default)]
    struct Runs {
        targets: BTreeSet<Id<Organization>>,
        finished: Mutex<Vec<(DateTime<Utc>, RunResult)>>,
        leasable: bool,
        cursor: Mutex<Option<Uuid>>,
        advanced: Mutex<Vec<(DateTime<Utc>, Uuid)>>,
        lease_intervals: Mutex<Vec<i64>>,
        lease_lost: bool,
        renewals: Mutex<u32>,
    }

    impl Runs {
        fn results(&self) -> Vec<RunResult> {
            self.finished
                .lock()
                .unwrap()
                .iter()
                .map(|(_, r)| r.clone())
                .collect()
        }
    }

    #[async_trait::async_trait]
    impl ExportRunStore for Runs {
        async fn target_orgs(&self) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
            Ok(self.targets.clone())
        }
        async fn due(&self, _: i64) -> Result<Vec<Id<Organization>>, RepositoryError> {
            Ok(self.targets.iter().copied().collect())
        }
        async fn try_lease(
            &self,
            org: Id<Organization>,
            _: Uuid,
            _: i64,
            interval_secs: i64,
        ) -> Result<Option<StoredExportTarget>, RepositoryError> {
            self.lease_intervals.lock().unwrap().push(interval_secs);
            Ok(self.leasable.then(|| StoredExportTarget {
                target: ExportTarget {
                    organization_id: org,
                    config: ExportTargetConfig {
                        enabled: true,
                        base_url: ExportUrl::service_root(
                            BASE_URL_FIELD,
                            "https://frost.example.org/v1.1",
                        )
                        .unwrap(),
                        auth: ExportAuth::None,
                    },
                    activated_at: generation(),
                },
                status: ExportTargetStatus::default(),
            }))
        }
        async fn renew_lease(
            &self,
            _: Id<Organization>,
            _: &ExportLease,
            _: i64,
        ) -> Result<bool, RepositoryError> {
            *self.renewals.lock().unwrap() += 1;
            Ok(!self.lease_lost)
        }
        async fn finish(
            &self,
            _: Id<Organization>,
            lease: &ExportLease,
            result: &RunResult,
        ) -> Result<(), RepositoryError> {
            self.finished
                .lock()
                .unwrap()
                .push((lease.generation, result.clone()));
            Ok(())
        }
        async fn cursor(&self, _: Id<Organization>) -> Result<Option<Uuid>, RepositoryError> {
            Ok(*self.cursor.lock().unwrap())
        }
        async fn advance_cursor(
            &self,
            _: Id<Organization>,
            lease: &ExportLease,
            cursor: Uuid,
        ) -> Result<(), RepositoryError> {
            *self.cursor.lock().unwrap() = Some(cursor);
            self.advanced
                .lock()
                .unwrap()
                .push((lease.generation, cursor));
            Ok(())
        }
    }

    /// Stands still until a test or the sink moves it.
    struct ManualClock(Mutex<DateTime<Utc>>);

    impl ManualClock {
        fn new() -> Self {
            Self(Mutex::new(generation()))
        }
        fn advance(&self, by: std::time::Duration) {
            let mut now = self.0.lock().unwrap();
            *now += chrono::TimeDelta::from_std(by).unwrap();
        }
    }

    impl domain::clock::Clock for ManualClock {
        fn now(&self) -> DateTime<Utc> {
            *self.0.lock().unwrap()
        }
    }

    /// Moves forward by `step` every time it is read.
    struct SteppingClock {
        now: Mutex<DateTime<Utc>>,
        step: chrono::TimeDelta,
    }

    impl domain::clock::Clock for SteppingClock {
        fn now(&self) -> DateTime<Utc> {
            let mut now = self.now.lock().unwrap();
            *now += self.step;
            *now
        }
    }

    #[derive(Default)]
    struct Readings {
        batches: Mutex<std::collections::VecDeque<ReadingBatch>>,
        afters: Mutex<Vec<Uuid>>,
        befores: Mutex<Vec<Uuid>>,
        limits: Mutex<Vec<i64>>,
        active: Mutex<Vec<(ModuleKey, Id<Organization>)>>,
        broken: bool,
    }

    #[async_trait::async_trait]
    impl ExportReadingSource for Readings {
        async fn readings_after(
            &self,
            active: &[(ModuleKey, Id<Organization>)],
            after: Uuid,
            before: Uuid,
            limit: i64,
        ) -> Result<ReadingBatch, RepositoryError> {
            if self.broken {
                return Err(RepositoryError::Internal(
                    "relation \"sensor_data\" does not exist".into(),
                ));
            }
            self.afters.lock().unwrap().push(after);
            self.befores.lock().unwrap().push(before);
            self.limits.lock().unwrap().push(limit);
            *self.active.lock().unwrap() = active.to_vec();
            Ok(self.batches.lock().unwrap().pop_front().unwrap_or_default())
        }
    }

    #[derive(Default)]
    struct Refs {
        map: Mutex<HashMap<String, ExportRef>>,
        generations: Mutex<BTreeSet<DateTime<Utc>>>,
    }

    #[async_trait::async_trait]
    impl ExportRefReader for Refs {
        async fn refs(
            &self,
            _: Id<Organization>,
        ) -> Result<HashMap<String, ExportRef>, RepositoryError> {
            Ok(self.map.lock().unwrap().clone())
        }
    }

    #[async_trait::async_trait]
    impl ExportRefWriter for Refs {
        async fn put_ref(
            &self,
            _: Id<Organization>,
            lease: &ExportLease,
            r: &ExportRef,
        ) -> Result<(), RepositoryError> {
            self.generations.lock().unwrap().insert(lease.generation);
            self.map.lock().unwrap().insert(r.ge_key.clone(), r.clone());
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Call {
        Create(String),
        Update(String),
        Move(String),
        Deleted(String),
        Observation(serde_json::Value),
        Batch(usize),
    }

    #[derive(Default)]
    struct Sink {
        calls: Arc<Mutex<Vec<Call>>>,
        reject_observations: bool,
        unreachable: bool,
        adopts_existing: bool,
        reject_sensor: bool,
        rejected_rows: u32,
        batch_error: Option<SinkError>,
        reject_raw_datastreams: bool,
        raw_streams: Arc<Mutex<Vec<String>>>,
        /// Each created raw datastream moves this clock, like a slow target would.
        slow_raw_streams: Option<(Arc<ManualClock>, std::time::Duration)>,
    }

    struct Session {
        calls: Arc<Mutex<Vec<Call>>>,
        raw_streams: Arc<Mutex<Vec<String>>>,
        reject_observations: bool,
        adopts_existing: bool,
        reject_sensor: bool,
        rejected_rows: u32,
        batch_error: Option<SinkError>,
        reject_raw_datastreams: bool,
        slow_raw_streams: Option<(Arc<ManualClock>, std::time::Duration)>,
    }

    #[async_trait::async_trait]
    impl SensorThingsSink for Sink {
        async fn open(&self, _: &ExportTargetConfig) -> Result<Box<dyn SinkSession>, SinkError> {
            if self.unreachable {
                return Err(SinkError::Unreachable("down".into()));
            }
            Ok(Box::new(Session {
                calls: self.calls.clone(),
                raw_streams: self.raw_streams.clone(),
                reject_observations: self.reject_observations,
                adopts_existing: self.adopts_existing,
                reject_sensor: self.reject_sensor,
                rejected_rows: self.rejected_rows,
                batch_error: self.batch_error.clone(),
                reject_raw_datastreams: self.reject_raw_datastreams,
                slow_raw_streams: self.slow_raw_streams.clone(),
            }))
        }
    }

    #[async_trait::async_trait]
    impl SinkSession for Session {
        async fn create_thing(&self, t: &ExportThing) -> Result<(RemoteId, bool), SinkError> {
            self.calls.lock().unwrap().push(Call::Create(t.key()));
            Ok((RemoteId(json!(t.key())), !self.adopts_existing))
        }
        async fn update_thing(&self, _: &RemoteId, t: &ExportThing) -> Result<(), SinkError> {
            self.calls.lock().unwrap().push(Call::Update(t.key()));
            Ok(())
        }
        async fn move_thing(&self, _: &RemoteId, t: &ExportThing) -> Result<(), SinkError> {
            self.calls.lock().unwrap().push(Call::Move(t.key()));
            Ok(())
        }
        async fn mark_thing_deleted(
            &self,
            id: &RemoteId,
            _: DateTime<Utc>,
        ) -> Result<(), SinkError> {
            self.calls
                .lock()
                .unwrap()
                .push(Call::Deleted(id.as_json().as_str().unwrap().to_string()));
            Ok(())
        }
        async fn ensure_sensor(&self) -> Result<RemoteId, SinkError> {
            if self.reject_sensor {
                return Err(SinkError::Rejected {
                    status: 400,
                    detail: "Authorization: Bearer secret".into(),
                });
            }
            Ok(RemoteId(json!(1)))
        }
        async fn ensure_observed_property(
            &self,
            _: &ExportProperty,
        ) -> Result<RemoteId, SinkError> {
            Ok(RemoteId(json!(2)))
        }
        async fn ensure_state_datastream(
            &self,
            s: &StateStream<'_>,
        ) -> Result<RemoteId, SinkError> {
            Ok(RemoteId(json!(s.ge_key)))
        }
        async fn post_observation(
            &self,
            _: &RemoteId,
            v: &StateValue,
            _: DateTime<Utc>,
        ) -> Result<(), SinkError> {
            if self.reject_observations {
                return Err(SinkError::Rejected {
                    status: 400,
                    detail: "no".into(),
                });
            }
            self.calls
                .lock()
                .unwrap()
                .push(Call::Observation(v.to_json()));
            Ok(())
        }
        async fn ensure_device_sensor(
            &self,
            sensor_id: &str,
            _: &str,
        ) -> Result<RemoteId, SinkError> {
            Ok(RemoteId(json!(sensor_id)))
        }
        async fn ensure_raw_datastream(&self, s: &RawStream<'_>) -> Result<RemoteId, SinkError> {
            self.raw_streams.lock().unwrap().push(s.name.to_string());
            if let Some((clock, by)) = &self.slow_raw_streams {
                clock.advance(*by);
            }
            if self.reject_raw_datastreams {
                return Err(SinkError::Rejected {
                    status: 400,
                    detail: "no".into(),
                });
            }
            Ok(RemoteId(json!(s.ge_key)))
        }
        async fn post_observation_batches(
            &self,
            batches: &[(RemoteId, Vec<ObservationRow>)],
        ) -> Result<BatchOutcome, SinkError> {
            let n = batches.iter().map(|(_, r)| r.len()).sum::<usize>();
            self.calls.lock().unwrap().push(Call::Batch(n));
            if let Some(e) = &self.batch_error {
                return Err(e.clone());
            }
            let n = u32::try_from(n).unwrap();
            Ok(BatchOutcome {
                accepted: n - self.rejected_rows,
                rejected: self.rejected_rows,
            })
        }
    }

    struct Catalog(Mutex<Vec<ExportThing>>);

    #[async_trait::async_trait]
    impl ExportCatalog for Catalog {
        async fn snapshot(
            &self,
            _: &BTreeSet<Id<Organization>>,
        ) -> Result<Vec<ExportThing>, RepositoryError> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    struct Activation(BTreeSet<Id<Organization>>);

    #[async_trait::async_trait]
    impl ModuleActivationReader for Activation {
        async fn own(
            &self,
            _: Id<Organization>,
        ) -> Result<BTreeMap<ModuleKey, OwnSwitch>, RepositoryError> {
            unimplemented!()
        }
        async fn resolve(
            &self,
            _: Id<Organization>,
        ) -> Result<BTreeMap<ModuleKey, ActivationResolution>, RepositoryError> {
            unimplemented!()
        }
        async fn active_orgs(
            &self,
            _: &ModuleKey,
        ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
            Ok(self.0.clone())
        }
    }

    struct Orgs;

    #[async_trait::async_trait]
    impl OrganizationReader for Orgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            unimplemented!()
        }
        async fn by_id(&self, _: Id<Organization>) -> Result<Organization, RepositoryError> {
            unimplemented!()
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs([
                (org(1), None),
                (org(2), Some(org(1))),
            ]))
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            unimplemented!()
        }
        async fn resource_counts(
            &self,
        ) -> Result<HashMap<Id<Organization>, OrganizationResourceCounts>, RepositoryError>
        {
            unimplemented!()
        }
    }

    fn tree(n: u128, status: &str) -> ExportThing {
        ExportThing {
            target: TargetRef::new("irrigation", "tree", Uuid::from_u128(n)).unwrap(),
            name: format!("Baum {n}"),
            description: "Linde".into(),
            attributes: BTreeMap::new(),
            coordinate: Some(Coordinate::new(54.79, 9.43).unwrap()),
            states: vec![ExportState {
                property: WATERING_STATUS,
                value: StateValue::Category(status.into()),
                observed_at: Utc::now(),
            }],
        }
    }

    fn reading(n: u128, thing: u128, ability: SensorAbilityName, value: f64) -> ExportReading {
        let unit = match ability {
            SensorAbilityName::Battery => SensorAbilityUnit::Volt,
            _ => SensorAbilityUnit::Percent,
        };
        ExportReading {
            reading_id: Uuid::from_u128(n),
            sensor_id: "eui-1".into(),
            sensor_model: "GES-1000".into(),
            target: TargetRef::new("irrigation", "tree", Uuid::from_u128(thing)).unwrap(),
            ability,
            unit,
            depth_cm: 40,
            value,
            quality_issue: None,
            measured_at: generation(),
        }
    }

    fn batch(rows: Vec<ExportReading>) -> ReadingBatch {
        let ids: BTreeSet<Uuid> = rows.iter().map(|r| r.reading_id).collect();
        ReadingBatch {
            scanned_until: ids.last().copied(),
            scanned: i64::try_from(ids.len()).unwrap(),
            rows,
        }
    }

    struct Fixture {
        sync: DataExportSync,
        sink_calls: Arc<Mutex<Vec<Call>>>,
        raw_streams: Arc<Mutex<Vec<String>>>,
        catalog: Arc<Catalog>,
        runs: Arc<Runs>,
        refs: Arc<Refs>,
        readings: Arc<Readings>,
    }

    struct Setup {
        things: Vec<ExportThing>,
        active: Vec<u128>,
        sink: Sink,
        refs: Arc<Refs>,
        leasable: bool,
        run_budget: std::time::Duration,
        readings: Vec<ReadingBatch>,
        batch_size: i64,
        reading_settle: std::time::Duration,
        broken_readings: bool,
        clock: Arc<dyn domain::clock::Clock>,
        lease_lost: bool,
    }

    impl Default for Setup {
        fn default() -> Self {
            Self {
                things: vec![],
                active: vec![1],
                sink: Sink::default(),
                refs: Arc::new(Refs::default()),
                leasable: true,
                run_budget: std::time::Duration::from_secs(60),
                readings: vec![],
                batch_size: 100,
                reading_settle: std::time::Duration::from_secs(120),
                broken_readings: false,
                clock: Arc::new(domain::clock::SystemClock),
                lease_lost: false,
            }
        }
    }

    fn fixture(things: Vec<ExportThing>, active: &[u128], sink: Sink) -> Fixture {
        fixture_with(things, active, sink, Arc::new(Refs::default()), true)
    }

    /// Shares `refs` so a second fixture continues where a first one left off.
    fn fixture_with(
        things: Vec<ExportThing>,
        active: &[u128],
        sink: Sink,
        refs: Arc<Refs>,
        leasable: bool,
    ) -> Fixture {
        build(Setup {
            things,
            active: active.to_vec(),
            sink,
            refs,
            leasable,
            ..Setup::default()
        })
    }

    fn fixture_budget(
        things: Vec<ExportThing>,
        active: &[u128],
        sink: Sink,
        refs: Arc<Refs>,
        leasable: bool,
        run_budget: std::time::Duration,
    ) -> Fixture {
        build(Setup {
            things,
            active: active.to_vec(),
            sink,
            refs,
            leasable,
            run_budget,
            ..Setup::default()
        })
    }

    fn build(setup: Setup) -> Fixture {
        let Setup {
            things,
            active,
            sink,
            refs,
            leasable,
            run_budget,
            readings,
            batch_size,
            reading_settle,
            broken_readings,
            clock,
            lease_lost,
        } = setup;
        let runs = Arc::new(Runs {
            targets: BTreeSet::from([org(1)]),
            leasable,
            lease_lost,
            ..Default::default()
        });
        let readings = Arc::new(Readings {
            batches: Mutex::new(readings.into()),
            broken: broken_readings,
            ..Default::default()
        });
        let catalog = Arc::new(Catalog(Mutex::new(things)));
        let registry = Arc::new(ExportRegistry::new());
        registry
            .install(BTreeMap::from([(
                ModuleKey::new("irrigation").unwrap(),
                catalog.clone() as Arc<dyn ExportCatalog>,
            )]))
            .unwrap();
        let sink_calls = sink.calls.clone();
        let raw_streams = sink.raw_streams.clone();
        let sync = DataExportSync::new(
            runs.clone(),
            refs.clone(),
            refs.clone(),
            readings.clone(),
            Arc::new(sink),
            registry,
            Arc::new(Activation(active.iter().map(|n| org(*n)).collect())),
            Arc::new(Orgs),
            clock,
            SyncSettings {
                interval_secs: 300,
                run_budget,
                batch_size,
                reading_settle,
            },
        );
        Fixture {
            sync,
            sink_calls,
            raw_streams,
            catalog,
            runs,
            refs,
            readings,
        }
    }

    fn calls(f: &Fixture) -> Vec<Call> {
        std::mem::take(&mut *f.sink_calls.lock().unwrap())
    }

    #[tokio::test]
    async fn the_first_pass_creates_things_and_writes_their_states() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(
            calls(&f),
            vec![
                Call::Create(tree(1, "good").key()),
                Call::Observation(json!("good"))
            ]
        );
        assert_eq!((stats.things_written, stats.observations_written), (1, 1));
        assert!(matches!(f.runs.results()[0], RunResult::Succeeded(_)));
    }

    #[tokio::test]
    async fn a_second_pass_without_changes_writes_nothing() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        calls(&f);

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(calls(&f).is_empty());
        assert_eq!(stats, ExportRunStats::default());
    }

    #[tokio::test]
    async fn a_changed_state_writes_one_observation() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        calls(&f);
        *f.catalog.0.lock().unwrap() = vec![tree(1, "bad")];

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(calls(&f), vec![Call::Observation(json!("bad"))]);
    }

    #[tokio::test]
    async fn renaming_updates_and_moving_posts_a_new_location() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        calls(&f);
        let mut changed = tree(1, "good");
        changed.name = "Baum eins".into();
        changed.coordinate = Some(Coordinate::new(54.80, 9.44).unwrap());
        *f.catalog.0.lock().unwrap() = vec![changed.clone()];

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            calls(&f),
            vec![Call::Update(changed.key()), Call::Move(changed.key())]
        );
    }

    #[tokio::test]
    async fn an_adopted_thing_gets_its_location_moved() {
        let f = fixture(
            vec![tree(1, "good")],
            &[1],
            Sink {
                adopts_existing: true,
                ..Default::default()
            },
        );

        f.sync.sync(org(1)).await.unwrap();

        let key = tree(1, "good").key();
        assert_eq!(
            calls(&f),
            vec![
                Call::Create(key.clone()),
                Call::Move(key),
                Call::Observation(json!("good"))
            ]
        );
    }

    #[tokio::test]
    async fn a_vanished_thing_is_marked_deleted_once() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        calls(&f);
        f.catalog.0.lock().unwrap().clear();

        f.sync.sync(org(1)).await.unwrap();
        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(calls(&f), vec![Call::Deleted(tree(1, "good").key())]);
    }

    #[tokio::test]
    async fn a_reappearing_thing_is_updated_again() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        f.catalog.0.lock().unwrap().clear();
        f.sync.sync(org(1)).await.unwrap();
        calls(&f);
        *f.catalog.0.lock().unwrap() = vec![tree(1, "good")];

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(calls(&f), vec![Call::Update(tree(1, "good").key())]);
        let key = tree(1, "good").key();
        assert!(!f.refs.map.lock().unwrap()[&key].deleted_marked);
    }

    #[tokio::test]
    async fn a_module_switched_off_in_the_whole_scope_marks_nothing() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());
        f.sync.sync(org(1)).await.unwrap();
        let off = fixture_with(vec![], &[], Sink::default(), f.refs.clone(), true);

        off.sync.sync(org(1)).await.unwrap();

        assert!(calls(&off).is_empty());
    }

    #[tokio::test]
    async fn a_rejected_observation_is_skipped_and_the_pass_succeeds() {
        let f = fixture(
            vec![tree(1, "good")],
            &[1],
            Sink {
                reject_observations: true,
                ..Default::default()
            },
        );

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(stats.skipped, 1);
        assert!(matches!(f.runs.results()[0], RunResult::Succeeded(_)));
    }

    #[tokio::test]
    async fn an_unreachable_target_fails_the_pass_with_backoff() {
        let f = fixture(
            vec![tree(1, "good")],
            &[1],
            Sink {
                unreachable: true,
                ..Default::default()
            },
        );

        assert!(f.sync.sync(org(1)).await.is_err());

        assert!(matches!(&f.runs.results()[0],
            RunResult::Failed { base_interval_secs: 300, error } if error.contains("unreachable")));
    }

    #[tokio::test]
    async fn a_rejected_sensor_fails_the_pass_without_the_remote_answer() {
        let f = fixture(
            vec![tree(1, "good")],
            &[1],
            Sink {
                reject_sensor: true,
                ..Default::default()
            },
        );

        assert!(f.sync.sync(org(1)).await.is_err());

        let RunResult::Failed { error, .. } = &f.runs.results()[0] else {
            panic!("the pass must fail");
        };
        assert!(error.contains("400"), "{error}");
        assert!(!error.contains("secret"), "{error}");
    }

    #[tokio::test]
    async fn a_target_held_by_another_replica_is_skipped() {
        let f = fixture_with(
            vec![tree(1, "good")],
            &[1],
            Sink::default(),
            Arc::new(Refs::default()),
            false,
        );

        assert_eq!(f.sync.sync(org(1)).await.unwrap(), None);
        assert!(calls(&f).is_empty());
        assert!(f.runs.results().is_empty());
    }

    #[tokio::test]
    async fn a_pass_that_lost_its_lease_stops_before_writing() {
        // Lease = budget + 60 s; the first renewal falls due after a third of it.
        let f = build(Setup {
            things: vec![tree(1, "good")],
            run_budget: std::time::Duration::from_secs(3600),
            clock: Arc::new(SteppingClock {
                now: Mutex::new(generation()),
                step: chrono::TimeDelta::seconds(1300),
            }),
            lease_lost: true,
            ..Setup::default()
        });

        let result = f.sync.sync(org(1)).await;

        assert!(matches!(result, Err(SyncError::LeaseLost)));
        assert!(calls(&f).is_empty());
        assert_eq!(*f.runs.renewals.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn a_long_pass_keeps_renewing_its_lease() {
        let f = build(Setup {
            things: vec![tree(1, "good"), tree(2, "bad")],
            run_budget: std::time::Duration::from_secs(3600),
            clock: Arc::new(SteppingClock {
                now: Mutex::new(generation()),
                step: chrono::TimeDelta::seconds(700),
            }),
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(*f.runs.renewals.lock().unwrap() >= 2);
    }

    #[tokio::test]
    async fn the_budget_stops_a_batch_before_the_next_new_stream() {
        let clock = Arc::new(ManualClock::new());
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
                reading(11, 1, SensorAbilityName::Battery, 3.6),
            ])],
            sink: Sink {
                slow_raw_streams: Some((clock.clone(), std::time::Duration::from_secs(61))),
                ..Sink::default()
            },
            clock,
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(f.raw_streams.lock().unwrap().len(), 1);
        assert_eq!(calls(&f).last(), Some(&Call::Batch(1)));
        assert_eq!(
            *f.runs.advanced.lock().unwrap(),
            vec![(generation(), Uuid::from_u128(10))],
            "the cursor stops before the reading that did not go out"
        );
    }

    #[tokio::test]
    async fn a_reading_cut_in_half_by_the_budget_waits_whole_for_the_next_pass() {
        let clock = Arc::new(ManualClock::new());
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![
                reading(10, 1, SensorAbilityName::Battery, 3.6),
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
            ])],
            sink: Sink {
                slow_raw_streams: Some((clock.clone(), std::time::Duration::from_secs(61))),
                ..Sink::default()
            },
            clock,
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(!calls(&f).iter().any(|c| matches!(c, Call::Batch(_))));
        assert!(f.runs.advanced.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn every_write_carries_the_leased_generation() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            *f.refs.generations.lock().unwrap(),
            BTreeSet::from([generation()])
        );
        assert_eq!(f.runs.finished.lock().unwrap()[0].0, generation());
        assert_eq!(
            *f.runs.advanced.lock().unwrap(),
            vec![(generation(), Uuid::from_u128(10))]
        );
    }

    #[tokio::test]
    async fn readings_of_an_exported_thing_go_out_as_one_batch_and_move_the_cursor() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
                reading(10, 1, SensorAbilityName::Battery, 3.6),
            ])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(
            calls(&f),
            vec![
                Call::Create(tree(1, "good").key()),
                Call::Observation(json!("good")),
                Call::Batch(2)
            ]
        );
        assert_eq!(stats.observations_written, 3);
        assert_eq!(stats.skipped, 0);
        assert_eq!(
            *f.readings.active.lock().unwrap(),
            vec![(ModuleKey::new("irrigation").unwrap(), org(1))]
        );
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
        let refs = f.refs.map.lock().unwrap();
        assert_eq!(refs["device:eui-1"].entity, ExportEntity::Sensor);
        let stream = raw_stream_key(&reading(10, 1, SensorAbilityName::Battery, 3.6));
        assert_eq!(refs[&stream].entity, ExportEntity::Datastream);
    }

    #[tokio::test]
    async fn readings_of_unknown_things_are_skipped_but_passed_by_the_cursor() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![reading(
                10,
                9,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(!calls(&f).iter().any(|c| matches!(c, Call::Batch(_))));
        assert_eq!(stats.skipped, 1);
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn a_thing_without_coordinate_gets_neither_states_nor_readings() {
        let mut unlocated = tree(1, "good");
        unlocated.coordinate = None;
        let f = build(Setup {
            things: vec![unlocated.clone()],
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(calls(&f), vec![Call::Create(unlocated.key())]);
        assert_eq!(stats.observations_written, 0);
        assert_eq!(stats.skipped, 1);
        let refs = f.refs.map.lock().unwrap();
        assert!(!refs.keys().any(|k| k.starts_with("state:")));
    }

    #[tokio::test]
    async fn the_cursor_starts_at_the_activation_when_unset() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            *f.readings.afters.lock().unwrap(),
            vec![v7_floor(generation())]
        );
        assert!(f.runs.advanced.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_stored_cursor_is_where_the_next_pass_continues() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            ..Setup::default()
        });
        *f.runs.cursor.lock().unwrap() = Some(Uuid::from_u128(7));

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(*f.readings.afters.lock().unwrap(), vec![Uuid::from_u128(7)]);
    }

    #[tokio::test]
    async fn full_batches_are_read_until_a_short_one() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![
                batch(vec![reading(10, 1, SensorAbilityName::SoilMoisture, 31.5)]),
                batch(vec![reading(11, 1, SensorAbilityName::SoilMoisture, 30.5)]),
            ],
            batch_size: 1,
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            *f.readings.afters.lock().unwrap(),
            vec![
                v7_floor(generation()),
                Uuid::from_u128(10),
                Uuid::from_u128(11)
            ]
        );
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(11)));
    }

    #[tokio::test]
    async fn a_batch_size_below_one_still_reads_one_reading_per_batch() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            batch_size: 0,
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(*f.readings.limits.lock().unwrap(), vec![1, 1]);
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn a_rejected_observation_row_counts_as_skipped() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            sink: Sink {
                rejected_rows: 1,
                ..Default::default()
            },
            readings: vec![batch(vec![
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
                reading(10, 1, SensorAbilityName::Battery, 3.6),
            ])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.observations_written, 2);
        assert!(matches!(f.runs.results()[0], RunResult::Succeeded(_)));
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn a_non_finite_value_is_skipped_instead_of_sent() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                f64::NAN,
            )])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(!calls(&f).iter().any(|c| matches!(c, Call::Batch(_))));
        assert_eq!(stats.skipped, 1);
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn value_less_readings_still_move_the_cursor() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![ReadingBatch {
                rows: vec![],
                scanned_until: Some(Uuid::from_u128(10)),
                scanned: 1,
            }],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(!calls(&f).iter().any(|c| matches!(c, Call::Batch(_))));
        assert_eq!(stats.skipped, 0);
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn a_failed_batch_post_keeps_the_cursor_and_fails_the_pass() {
        for error in [
            SinkError::Unreachable("down".into()),
            SinkError::Rejected {
                status: 413,
                detail: "too large".into(),
            },
        ] {
            let f = build(Setup {
                things: vec![tree(1, "good")],
                sink: Sink {
                    batch_error: Some(error),
                    ..Default::default()
                },
                readings: vec![batch(vec![reading(
                    10,
                    1,
                    SensorAbilityName::SoilMoisture,
                    31.5,
                )])],
                ..Setup::default()
            });

            assert!(f.sync.sync(org(1)).await.is_err());

            assert_eq!(*f.runs.cursor.lock().unwrap(), None);
            assert!(matches!(f.runs.results()[0], RunResult::Failed { .. }));
        }
    }

    #[tokio::test]
    async fn a_rejected_raw_datastream_skips_its_reading_and_stores_no_ref() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            sink: Sink {
                reject_raw_datastreams: true,
                ..Default::default()
            },
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(stats.skipped, 1);
        assert!(!calls(&f).iter().any(|c| matches!(c, Call::Batch(_))));
        let refs = f.refs.map.lock().unwrap();
        assert!(!refs.keys().any(|k| k.starts_with("raw:")));
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(10)));
    }

    #[tokio::test]
    async fn a_known_raw_datastream_is_reused_on_the_next_pass() {
        let first = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });
        first.sync.sync(org(1)).await.unwrap();
        let second = build(Setup {
            things: vec![tree(1, "good")],
            sink: Sink {
                reject_raw_datastreams: true,
                ..Default::default()
            },
            refs: first.refs.clone(),
            readings: vec![batch(vec![reading(
                11,
                1,
                SensorAbilityName::SoilMoisture,
                30.5,
            )])],
            ..Setup::default()
        });

        let stats = second.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(calls(&second), vec![Call::Batch(1)]);
        assert_eq!((stats.observations_written, stats.skipped), (1, 0));
    }

    #[tokio::test]
    async fn no_raw_phase_after_an_exhausted_budget() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            run_budget: std::time::Duration::ZERO,
            readings: vec![batch(vec![reading(
                10,
                1,
                SensorAbilityName::SoilMoisture,
                31.5,
            )])],
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert!(f.readings.afters.lock().unwrap().is_empty());
        assert_eq!(*f.runs.cursor.lock().unwrap(), None);
    }

    #[tokio::test]
    async fn an_exhausted_budget_leaves_the_things_for_the_next_pass() {
        let f = fixture_budget(
            vec![tree(1, "good")],
            &[1],
            Sink::default(),
            Arc::new(Refs::default()),
            true,
            std::time::Duration::ZERO,
        );

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert!(calls(&f).is_empty());
        assert_eq!(stats, ExportRunStats::default());
        assert!(matches!(f.runs.results()[0], RunResult::Succeeded(_)));
    }

    #[tokio::test]
    async fn readings_are_read_only_below_the_settle_bound() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            reading_settle: std::time::Duration::from_secs(120),
            ..Setup::default()
        });
        let settle = chrono::Duration::seconds(120);
        let earliest = v7_floor(Utc::now() - settle);

        f.sync.sync(org(1)).await.unwrap();

        let latest = v7_floor(Utc::now() - settle);
        let befores = f.readings.befores.lock().unwrap();
        assert_eq!(befores.len(), 1);
        assert!(
            (earliest..=latest).contains(&befores[0]),
            "{} not in {earliest}..={latest}",
            befores[0]
        );
    }

    #[tokio::test]
    async fn a_rejected_raw_datastream_is_asked_once_per_pass() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            sink: Sink {
                reject_raw_datastreams: true,
                ..Default::default()
            },
            readings: vec![batch(vec![
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
                reading(11, 1, SensorAbilityName::SoilMoisture, 30.5),
                reading(12, 1, SensorAbilityName::SoilMoisture, 29.5),
            ])],
            ..Setup::default()
        });

        let stats = f.sync.sync(org(1)).await.unwrap().unwrap();

        assert_eq!(f.raw_streams.lock().unwrap().len(), 1);
        assert_eq!(stats.skipped, 3);
        assert_eq!(*f.runs.cursor.lock().unwrap(), Some(Uuid::from_u128(12)));
    }

    #[tokio::test]
    async fn a_raw_datastream_without_depth_names_no_depth() {
        let mut battery = reading(10, 1, SensorAbilityName::Battery, 3.6);
        battery.depth_cm = 0;
        let f = build(Setup {
            things: vec![tree(1, "good")],
            readings: vec![batch(vec![
                battery,
                reading(10, 1, SensorAbilityName::SoilMoisture, 31.5),
            ])],
            ..Setup::default()
        });

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            *f.raw_streams.lock().unwrap(),
            vec![
                "Baum 1 · Batteriespannung".to_string(),
                "Baum 1 · Bodenfeuchte 40 cm".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn a_database_failure_is_recorded_without_its_detail() {
        let f = build(Setup {
            things: vec![tree(1, "good")],
            broken_readings: true,
            ..Setup::default()
        });

        assert!(f.sync.sync(org(1)).await.is_err());

        let RunResult::Failed { error, .. } = &f.runs.results()[0] else {
            panic!("the pass must fail");
        };
        assert_eq!(error, "internal error");
    }

    #[tokio::test]
    async fn the_lease_is_asked_with_the_interval() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(*f.runs.lease_intervals.lock().unwrap(), vec![300]);
    }

    #[tokio::test]
    async fn run_due_syncs_every_due_organization() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());

        f.sync.run_due().await;

        assert_eq!(f.runs.results().len(), 1);
    }
}
