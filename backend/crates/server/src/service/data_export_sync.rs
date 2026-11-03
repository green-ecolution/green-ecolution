//! One export pass per organization: the modules' current objects are diffed
//! against the refs of the last pass, and only the differences are written.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use domain::{
    Id, RepositoryError,
    data_export::{
        ExportEntity, ExportProperty, ExportRef, ExportRefReader, ExportRefWriter, ExportRunStats,
        ExportRunStore, ExportState, ExportTargetConfig, ExportThing, RemoteId, RunResult,
        SENSOR_KEY, SensorThingsSink, SinkError, SinkSession, StateStream, export_scope,
        location_key, property_key, state_stream_key,
    },
    module::{ModuleActivationReader, ModuleKey},
    organization::{Organization, OrganizationReader},
};

use super::export_registry::ExportRegistry;

/// Headroom between the run budget and the lease, so a pass that stops at its
/// budget still finishes inside its lease.
const LEASE_MARGIN_SECS: i64 = 60;

pub struct SyncSettings {
    pub interval_secs: i64,
    pub run_budget: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Sink(#[from] SinkError),
}

pub struct DataExportSync {
    runs: Arc<dyn ExportRunStore>,
    refs: Arc<dyn ExportRefReader>,
    ref_writer: Arc<dyn ExportRefWriter>,
    sink: Arc<dyn SensorThingsSink>,
    registry: Arc<ExportRegistry>,
    activation: Arc<dyn ModuleActivationReader>,
    organizations: Arc<dyn OrganizationReader>,
    settings: SyncSettings,
}

impl DataExportSync {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runs: Arc<dyn ExportRunStore>,
        refs: Arc<dyn ExportRefReader>,
        ref_writer: Arc<dyn ExportRefWriter>,
        sink: Arc<dyn SensorThingsSink>,
        registry: Arc<ExportRegistry>,
        activation: Arc<dyn ModuleActivationReader>,
        organizations: Arc<dyn OrganizationReader>,
        settings: SyncSettings,
    ) -> Self {
        Self {
            runs,
            refs,
            ref_writer,
            sink,
            registry,
            activation,
            organizations,
            settings,
        }
    }

    pub async fn run_due(&self) {
        let due = match self.runs.due().await {
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
        let Some(leased) = self.runs.try_lease(org, self.lease_secs()).await? else {
            return Ok(None);
        };
        let generation = leased.target.activated_at;
        let outcome = self.pass(org, generation, &leased.target.config).await;
        let result = match &outcome {
            Ok(stats) => RunResult::Succeeded(*stats),
            Err(e) => RunResult::Failed {
                error: e.to_string(),
                base_interval_secs: self.settings.interval_secs,
            },
        };
        let finished = self.runs.finish(org, generation, &result).await;
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
        generation: DateTime<Utc>,
        config: &ExportTargetConfig,
    ) -> Result<ExportRunStats, SyncError> {
        let started = Instant::now();
        let hierarchy = self.organizations.hierarchy().await?;
        let targets = self.runs.target_orgs().await?;
        let scope = export_scope(&hierarchy, &targets, org);
        let session = self.sink.open(config).await?;
        let mut pass = Pass {
            org,
            generation,
            session,
            refs: self.refs.refs(org).await?,
            writer: self.ref_writer.as_ref(),
            stats: ExportRunStats::default(),
            started,
            budget: self.settings.run_budget,
        };
        let sensor = pass.sensor().await?;
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
            pass.module(&module, &things, &sensor).await?;
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
    /// The leased target's `activated_at`; the stores drop writes of a stale pass.
    generation: DateTime<Utc>,
    session: Box<dyn SinkSession>,
    refs: HashMap<String, ExportRef>,
    writer: &'a dyn ExportRefWriter,
    stats: ExportRunStats,
    started: Instant,
    budget: Duration,
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
        self.started.elapsed() >= self.budget
    }

    async fn remember(&mut self, export_ref: ExportRef) -> Result<(), RepositoryError> {
        self.writer
            .put_ref(self.org, self.generation, &export_ref)
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
            let marked = self
                .session
                .mark_thing_deleted(&thing.remote_id, Utc::now())
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
        shared::coordinates::Coordinate,
        target::TargetRef,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

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
        async fn due(&self) -> Result<Vec<Id<Organization>>, RepositoryError> {
            Ok(self.targets.iter().copied().collect())
        }
        async fn try_lease(
            &self,
            org: Id<Organization>,
            _: i64,
        ) -> Result<Option<StoredExportTarget>, RepositoryError> {
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
        async fn finish(
            &self,
            _: Id<Organization>,
            generation: DateTime<Utc>,
            result: &RunResult,
        ) -> Result<(), RepositoryError> {
            self.finished
                .lock()
                .unwrap()
                .push((generation, result.clone()));
            Ok(())
        }
        async fn cursor(&self, _: Id<Organization>) -> Result<Option<uuid::Uuid>, RepositoryError> {
            Ok(None)
        }
        async fn advance_cursor(
            &self,
            _: Id<Organization>,
            _: DateTime<Utc>,
            _: uuid::Uuid,
        ) -> Result<(), RepositoryError> {
            Ok(())
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
            generation: DateTime<Utc>,
            r: &ExportRef,
        ) -> Result<(), RepositoryError> {
            self.generations.lock().unwrap().insert(generation);
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
    }

    #[derive(Default)]
    struct Sink {
        calls: Arc<Mutex<Vec<Call>>>,
        reject_observations: bool,
        unreachable: bool,
        adopts_existing: bool,
        reject_sensor: bool,
    }

    struct Session {
        calls: Arc<Mutex<Vec<Call>>>,
        reject_observations: bool,
        adopts_existing: bool,
        reject_sensor: bool,
    }

    #[async_trait::async_trait]
    impl SensorThingsSink for Sink {
        async fn open(&self, _: &ExportTargetConfig) -> Result<Box<dyn SinkSession>, SinkError> {
            if self.unreachable {
                return Err(SinkError::Unreachable("down".into()));
            }
            Ok(Box::new(Session {
                calls: self.calls.clone(),
                reject_observations: self.reject_observations,
                adopts_existing: self.adopts_existing,
                reject_sensor: self.reject_sensor,
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

    struct Fixture {
        sync: DataExportSync,
        sink_calls: Arc<Mutex<Vec<Call>>>,
        catalog: Arc<Catalog>,
        runs: Arc<Runs>,
        refs: Arc<Refs>,
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
        fixture_budget(
            things,
            active,
            sink,
            refs,
            leasable,
            std::time::Duration::from_secs(60),
        )
    }

    fn fixture_budget(
        things: Vec<ExportThing>,
        active: &[u128],
        sink: Sink,
        refs: Arc<Refs>,
        leasable: bool,
        run_budget: std::time::Duration,
    ) -> Fixture {
        let runs = Arc::new(Runs {
            targets: BTreeSet::from([org(1)]),
            leasable,
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
        let sync = DataExportSync::new(
            runs.clone(),
            refs.clone(),
            refs.clone(),
            Arc::new(sink),
            registry,
            Arc::new(Activation(active.iter().map(|n| org(*n)).collect())),
            Arc::new(Orgs),
            SyncSettings {
                interval_secs: 300,
                run_budget,
            },
        );
        Fixture {
            sync,
            sink_calls,
            catalog,
            runs,
            refs,
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
    async fn every_write_carries_the_leased_generation() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());

        f.sync.sync(org(1)).await.unwrap();

        assert_eq!(
            *f.refs.generations.lock().unwrap(),
            BTreeSet::from([generation()])
        );
        assert_eq!(f.runs.finished.lock().unwrap()[0].0, generation());
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
    async fn run_due_syncs_every_due_organization() {
        let f = fixture(vec![tree(1, "good")], &[1], Sink::default());

        f.sync.run_due().await;

        assert_eq!(f.runs.results().len(), 1);
    }
}
