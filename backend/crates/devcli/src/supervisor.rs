use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        Mutex as AsyncMutex,
        broadcast::{self, error::RecvError},
    },
    task::JoinHandle,
};

use crate::{
    graph::Graph,
    logs::LogStore,
    sync::{lock, read, write},
    units::{Origin, Unit, UnitKind},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum UnitState {
    Stopped,
    Waiting { on: Vec<String> },
    Starting,
    Ready,
    Done,
    Failed { reason: String },
}

impl UnitState {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Ready | Self::Done)
    }

    pub fn short(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Waiting { .. } => "waiting",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Done => "done",
            Self::Failed { .. } => "failed",
        }
    }
}

impl fmt::Display for UnitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Waiting { on } => write!(f, "waiting for {}", on.join(", ")),
            Self::Failed { reason } => write!(f, "failed: {reason}"),
            other => f.write_str(other.short()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitStatus {
    pub name: String,
    pub kind: UnitKind,
    pub origin: Origin,
    pub state: UnitState,
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileChange {
    pub profile: String,
    pub stopped: Vec<String>,
    pub restarted: Vec<String>,
    pub started: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SupervisorError {
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    #[error("the supervisor is shutting down")]
    ShuttingDown,
}

/// How often `stop` and `restart` re-resolve an entry a concurrent profile switch retired.
const RESOLVE_ATTEMPTS: usize = 3;

pub struct Supervisor {
    entries: RwLock<BTreeMap<String, Arc<Entry>>>,
    graph: RwLock<Arc<Graph>>,
    logs: Arc<LogStore>,
    events: broadcast::Sender<UnitStatus>,
    profile: Mutex<String>,
    profiles: broadcast::Sender<String>,
    /// Serialises profile switches so two `apply` calls never interleave their diffs.
    switching: AsyncMutex<()>,
    /// Set first thing in `stop_all`; no lifecycle is spawned afterwards.
    closing: AtomicBool,
}

struct Entry {
    unit: Arc<dyn Unit>,
    status: Mutex<UnitStatus>,
    task: Mutex<Option<JoinHandle<()>>>,
    /// Held across a whole start/stop/restart so control operations on one unit never overlap.
    control: AsyncMutex<()>,
    /// Set under `control` once a profile switch stopped and replaced this entry, so a
    /// start that fetched it before the switch cannot revive a unit the profile dropped.
    retired: AtomicBool,
}

impl Entry {
    fn new(unit: Arc<dyn Unit>) -> Self {
        let status = UnitStatus {
            name: unit.name().to_owned(),
            kind: unit.kind(),
            origin: unit.origin(),
            state: UnitState::Stopped,
            since: Utc::now(),
        };
        Self {
            unit,
            status: Mutex::new(status),
            task: Mutex::new(None),
            control: AsyncMutex::new(()),
            retired: AtomicBool::new(false),
        }
    }
}

impl Supervisor {
    /// `graph` must contain exactly the given units; `assemble` guarantees that.
    pub fn new(units: Vec<Arc<dyn Unit>>, graph: Graph, logs: Arc<LogStore>) -> Arc<Self> {
        let (events, _) = broadcast::channel(1024);
        let (profiles, _) = broadcast::channel(16);
        let entries = units
            .into_iter()
            .map(|unit| (unit.name().to_owned(), Arc::new(Entry::new(unit))))
            .collect();
        Arc::new(Self {
            entries: RwLock::new(entries),
            graph: RwLock::new(Arc::new(graph)),
            logs,
            events,
            profile: Mutex::new(String::new()),
            profiles,
            switching: AsyncMutex::new(()),
            closing: AtomicBool::new(false),
        })
    }

    pub fn logs(&self) -> &Arc<LogStore> {
        &self.logs
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UnitStatus> {
        self.events.subscribe()
    }

    pub fn profile(&self) -> String {
        lock(&self.profile).clone()
    }

    pub fn set_profile(&self, name: &str) {
        name.clone_into(&mut lock(&self.profile));
        let _ = self.profiles.send(name.to_owned());
    }

    pub fn subscribe_profile(&self) -> broadcast::Receiver<String> {
        self.profiles.subscribe()
    }

    pub fn status(&self) -> Vec<UnitStatus> {
        let graph = self.graph();
        let entries = read(&self.entries);
        graph
            .start_order()
            .iter()
            .filter_map(|name| entries.get(name))
            .map(|entry| lock(&entry.status).clone())
            .collect()
    }

    pub async fn start_all(self: &Arc<Self>) {
        let graph = self.graph();
        for name in graph.start_order() {
            // Fails only once shutdown began, when nothing should start anyway.
            let _ = self.start(name).await;
        }
    }

    /// No-op while the unit's lifecycle is still running (including a finished oneshot).
    /// Fails with `ShuttingDown` once `stop_all` began.
    pub async fn start(self: &Arc<Self>, name: &str) -> Result<(), SupervisorError> {
        let entry = self.entry(name)?;
        let _control = entry.control.lock().await;
        let running = lock(&entry.task)
            .as_ref()
            .is_some_and(|task| !task.is_finished());
        if running {
            return Ok(());
        }
        self.spawn_lifecycle(&entry).await
    }

    pub async fn stop(&self, name: &str) -> Result<(), SupervisorError> {
        for attempt in 1..=RESOLVE_ATTEMPTS {
            let entry = self.entry(name)?;
            let control = entry.control.lock().await;
            if entry.retired.load(Ordering::SeqCst) && attempt < RESOLVE_ATTEMPTS {
                drop(control);
                self.await_switch().await;
                continue;
            }
            self.stop_entry(&entry).await;
            break;
        }
        Ok(())
    }

    pub async fn stop_all(&self) {
        self.closing.store(true, Ordering::SeqCst);
        let graph = self.graph();
        let affected: Vec<Arc<Entry>> = {
            let entries = read(&self.entries);
            graph
                .start_order()
                .iter()
                .rev()
                .filter_map(|name| entries.get(name).map(Arc::clone))
                .collect()
        };
        for entry in affected {
            let _control = entry.control.lock().await;
            self.stop_entry(&entry).await;
        }
    }

    /// Fails with `ShuttingDown` once `stop_all` began; units it already stopped stay stopped.
    pub async fn restart(
        self: &Arc<Self>,
        name: &str,
        cascade: bool,
    ) -> Result<(), SupervisorError> {
        for attempt in 1..=RESOLVE_ATTEMPTS {
            let graph = self.graph();
            let dependents = if cascade {
                graph.transitive_dependents(name)
            } else {
                Vec::new()
            };
            let affected: Vec<(&String, Arc<Entry>)> = {
                let entries = read(&self.entries);
                graph
                    .start_order()
                    .iter()
                    .filter(|unit| *unit == name || dependents.contains(unit))
                    .filter_map(|unit| entries.get(unit).map(|entry| (unit, Arc::clone(entry))))
                    .collect()
            };
            let Some(entry) = affected
                .iter()
                .find(|(unit, _)| *unit == name)
                .map(|(_, entry)| Arc::clone(entry))
            else {
                return Err(SupervisorError::UnknownUnit(name.to_owned()));
            };
            let dependents: Vec<&Arc<Entry>> = affected
                .iter()
                .filter(|(unit, _)| *unit != name)
                .map(|(_, entry)| entry)
                .collect();
            // Taking several control locks only ever in start order rules out deadlocks.
            let mut guards = Vec::with_capacity(affected.len());
            for (_, entry) in &affected {
                guards.push(entry.control.lock().await);
            }
            let stale = affected
                .iter()
                .any(|(_, entry)| entry.retired.load(Ordering::SeqCst));
            if stale && attempt < RESOLVE_ATTEMPTS {
                drop(guards);
                self.await_switch().await;
                continue;
            }
            if self.is_closing() {
                return Err(SupervisorError::ShuttingDown);
            }
            for dependent in dependents.iter().rev() {
                self.stop_entry(dependent).await;
            }
            self.stop_entry(&entry).await;
            self.spawn_lifecycle(&entry).await?;
            for dependent in dependents {
                self.spawn_lifecycle(dependent).await?;
            }
            break;
        }
        Ok(())
    }

    /// Stops units the new profile drops, restarts those whose definition changed, starts
    /// new ones; everything else keeps running. Failed units are started again.
    pub async fn apply(
        self: &Arc<Self>,
        profile: &str,
        units: Vec<Arc<dyn Unit>>,
        graph: Graph,
    ) -> Result<ProfileChange, SupervisorError> {
        let _switching = self.switching.lock().await;
        if self.is_closing() {
            return Err(SupervisorError::ShuttingDown);
        }
        let old_graph = self.graph();
        let current = read(&self.entries).clone();
        let incoming: BTreeMap<String, Arc<dyn Unit>> = units
            .into_iter()
            .map(|unit| (unit.name().to_owned(), unit))
            .collect();
        let mut change = ProfileChange {
            profile: profile.to_owned(),
            ..ProfileChange::default()
        };
        // Fingerprints leave out dependencies, so a changed dependency set counts as a change too.
        let unchanged = |name: &str, entry: &Entry, unit: &Arc<dyn Unit>| {
            unit.definition() == entry.unit.definition()
                && old_graph
                    .dependencies(name)
                    .collect::<BTreeSet<_>>()
                    .eq(&graph.dependencies(name).collect::<BTreeSet<_>>())
        };
        for name in old_graph.start_order().iter().rev() {
            let Some(entry) = current.get(name) else {
                continue;
            };
            let kept = incoming
                .get(name)
                .is_some_and(|unit| unchanged(name, entry, unit));
            if kept {
                continue;
            }
            let _control = entry.control.lock().await;
            self.stop_entry(entry).await;
            entry.retired.store(true, Ordering::SeqCst);
            if incoming.contains_key(name) {
                change.restarted.push(name.clone());
            } else {
                change.stopped.push(name.clone());
            }
        }
        let mut entries = BTreeMap::new();
        for (name, unit) in incoming {
            let entry = match current.get(&name) {
                Some(entry) if unchanged(&name, entry, &unit) => Arc::clone(entry),
                Some(_) => Arc::new(Entry::new(unit)),
                None => {
                    change.started.push(name.clone());
                    Arc::new(Entry::new(unit))
                }
            };
            entries.insert(name, entry);
        }
        *write(&self.entries) = entries;
        *write(&self.graph) = Arc::new(graph);
        self.set_profile(profile);
        self.start_all().await;
        Ok(change)
    }

    fn graph(&self) -> Arc<Graph> {
        Arc::clone(&read(&self.graph))
    }

    fn entry(&self, name: &str) -> Result<Arc<Entry>, SupervisorError> {
        read(&self.entries)
            .get(name)
            .map(Arc::clone)
            .ok_or_else(|| SupervisorError::UnknownUnit(name.to_owned()))
    }

    /// The switch that retired an entry installs its successor before releasing `switching`.
    /// The caller must hold no control lock: `apply` takes them while holding `switching`.
    async fn await_switch(&self) {
        drop(self.switching.lock().await);
    }

    fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    /// The caller holds `entry.control`. Checking `closing` under that lock guarantees that
    /// `stop_all` either sees the new lifecycle or no lifecycle is spawned.
    async fn spawn_lifecycle(self: &Arc<Self>, entry: &Arc<Entry>) -> Result<(), SupervisorError> {
        if lock(&entry.task).is_some() {
            // A displaced lifecycle may still own a process; stop it instead of orphaning it.
            self.stop_entry(entry).await;
        }
        if self.is_closing() {
            return Err(SupervisorError::ShuttingDown);
        }
        // The profile switch that retired the entry starts its successor, if there is one.
        if entry.retired.load(Ordering::SeqCst) {
            return Ok(());
        }
        let this = Arc::clone(self);
        let owned = Arc::clone(entry);
        let handle = tokio::spawn(async move { this.lifecycle(&owned).await });
        *lock(&entry.task) = Some(handle);
        Ok(())
    }

    async fn lifecycle(&self, entry: &Entry) {
        let name = entry.unit.name();
        // Subscribe before the first check so no dependency change slips through.
        let mut events = self.events.subscribe();
        loop {
            let graph = self.graph();
            let pending: Vec<String> = graph
                .dependencies(name)
                .filter(|dep| !self.state_of(dep).is_some_and(|state| state.is_satisfied()))
                .map(str::to_owned)
                .collect();
            if pending.is_empty() {
                break;
            }
            self.set_state(entry, UnitState::Waiting { on: pending });
            if let Err(RecvError::Closed) = events.recv().await {
                return;
            }
        }
        self.set_state(entry, UnitState::Starting);
        match entry.unit.start(Arc::clone(&self.logs)).await {
            Ok(running) => {
                let settled = match entry.unit.kind() {
                    UnitKind::Service => UnitState::Ready,
                    UnitKind::Oneshot => UnitState::Done,
                };
                self.set_state(entry, settled);
                let reason = running.exited.await;
                self.set_state(entry, UnitState::Failed { reason });
            }
            Err(err) => self.set_state(
                entry,
                UnitState::Failed {
                    reason: err.to_string(),
                },
            ),
        }
    }

    async fn stop_entry(&self, entry: &Entry) {
        let task = lock(&entry.task).take();
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
        }
        let state = match entry.unit.stop().await {
            Ok(()) => UnitState::Stopped,
            Err(err) => UnitState::Failed {
                reason: format!("stop failed: {err}"),
            },
        };
        self.set_state(entry, state);
    }

    fn state_of(&self, name: &str) -> Option<UnitState> {
        read(&self.entries)
            .get(name)
            .map(|entry| lock(&entry.status).state.clone())
    }

    fn set_state(&self, entry: &Entry, state: UnitState) {
        let snapshot = {
            let mut status = lock(&entry.status);
            if status.state == state {
                return;
            }
            status.state = state;
            status.since = Utc::now();
            status.clone()
        };
        let _ = self.events.send(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::units::fake::{FakeUnit, Journal};

    fn setup(units: Vec<Arc<FakeUnit>>, edges: &[(&str, &[&str])]) -> Arc<Supervisor> {
        let graph = Graph::new(edges.iter().map(|(unit, deps)| {
            (
                (*unit).to_owned(),
                deps.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>(),
            )
        }))
        .unwrap();
        let units = units.into_iter().map(|u| u as Arc<dyn Unit>).collect();
        Supervisor::new(units, graph, LogStore::new(100))
    }

    async fn settle(sup: &Supervisor, done: impl Fn(&[UnitStatus]) -> bool) {
        tokio::time::timeout(Duration::from_secs(60), async {
            while !done(&sup.status()) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("supervisor did not settle");
    }

    fn all_satisfied(statuses: &[UnitStatus]) -> bool {
        statuses.iter().all(|s| s.state.is_satisfied())
    }

    fn state(sup: &Supervisor, name: &str) -> UnitState {
        sup.status()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap()
            .state
    }

    #[tokio::test(start_paused = true)]
    async fn starts_units_after_their_dependencies() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("backend", &journal)),
                Arc::new(FakeUnit::oneshot("migrations", &journal)),
                Arc::new(FakeUnit::service("db", &journal).ready_after(Duration::from_secs(5))),
            ],
            &[
                ("db", &[]),
                ("migrations", &["db"]),
                ("backend", &["migrations"]),
            ],
        );

        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        assert_eq!(
            journal.entries(),
            ["start db", "start migrations", "start backend"]
        );
        assert_eq!(state(&sup, "migrations"), UnitState::Done);
        assert_eq!(state(&sup, "backend"), UnitState::Ready);
    }

    #[tokio::test(start_paused = true)]
    async fn dependents_keep_waiting_while_a_dependency_failed() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal).failing("port in use")),
                Arc::new(FakeUnit::service("backend", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"])],
        );

        sup.start_all().await;
        settle(&sup, |s| matches!(s[0].state, UnitState::Failed { .. })).await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        assert_eq!(
            state(&sup, "db"),
            UnitState::Failed {
                reason: "port in use".into()
            }
        );
        assert_eq!(
            state(&sup, "backend"),
            UnitState::Waiting {
                on: vec!["db".into()]
            }
        );
        assert_eq!(journal.entries(), ["start db"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_crashing_service_becomes_failed() {
        let journal = Journal::default();
        let db = Arc::new(FakeUnit::service("db", &journal));
        let sup = setup(vec![db.clone()], &[("db", &[])]);

        sup.start_all().await;
        settle(&sup, all_satisfied).await;
        db.crash("killed by OOM");
        settle(&sup, |s| !all_satisfied(s)).await;

        assert_eq!(
            state(&sup, "db"),
            UnitState::Failed {
                reason: "killed by OOM".into()
            }
        );
    }

    #[tokio::test(start_paused = true)]
    async fn stop_all_runs_in_reverse_start_order() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("backend", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"])],
        );

        sup.start_all().await;
        settle(&sup, all_satisfied).await;
        sup.stop_all().await;

        assert_eq!(&journal.entries()[2..], ["stop backend", "stop db"]);
        assert!(sup.status().iter().all(|s| s.state == UnitState::Stopped));
    }

    #[tokio::test(start_paused = true)]
    async fn restart_without_cascade_leaves_dependents_running() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("backend", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        sup.restart("db", false).await.unwrap();
        settle(&sup, all_satisfied).await;

        assert_eq!(&journal.entries()[2..], ["stop db", "start db"]);
    }

    #[tokio::test(start_paused = true)]
    async fn restart_with_cascade_restarts_dependents_after_the_unit() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("backend", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        sup.restart("db", true).await.unwrap();
        settle(&sup, all_satisfied).await;

        assert_eq!(
            &journal.entries()[2..],
            ["stop backend", "stop db", "start db", "start backend"]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn stop_and_start_a_single_unit() {
        let journal = Journal::default();
        let sup = setup(
            vec![Arc::new(FakeUnit::service("db", &journal))],
            &[("db", &[])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        sup.stop("db").await.unwrap();
        assert_eq!(state(&sup, "db"), UnitState::Stopped);
        sup.start("db").await.unwrap();
        settle(&sup, all_satisfied).await;

        assert_eq!(journal.entries(), ["start db", "stop db", "start db"]);
    }

    #[tokio::test(start_paused = true)]
    async fn unknown_units_are_rejected() {
        let sup = setup(vec![], &[]);
        assert_eq!(
            sup.restart("nope", false).await,
            Err(SupervisorError::UnknownUnit("nope".into()))
        );
    }

    fn slow_stopping_db(journal: &Journal) -> Arc<Supervisor> {
        setup(
            vec![Arc::new(
                FakeUnit::service("db", journal).stop_after(Duration::from_secs(5)),
            )],
            &[("db", &[])],
        )
    }

    #[tokio::test(start_paused = true)]
    async fn a_restart_racing_stop_all_starts_nothing_afterwards() {
        let journal = Journal::default();
        let sup = slow_stopping_db(&journal);
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        let restart = tokio::spawn({
            let sup = Arc::clone(&sup);
            async move { sup.restart("db", false).await }
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        sup.stop_all().await;
        let restarted = restart.await.unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;

        assert_eq!(restarted, Err(SupervisorError::ShuttingDown));
        assert_eq!(sup.start("db").await, Err(SupervisorError::ShuttingDown));
        assert_eq!(journal.entries(), ["start db", "stop db", "stop db"]);
        assert_eq!(state(&sup, "db"), UnitState::Stopped);
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_starts_start_the_unit_once() {
        let journal = Journal::default();
        let sup = setup(
            vec![Arc::new(FakeUnit::service("db", &journal))],
            &[("db", &[])],
        );

        let (first, second) = tokio::join!(sup.start("db"), sup.start("db"));
        first.unwrap();
        second.unwrap();
        settle(&sup, all_satisfied).await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        assert_eq!(journal.entries(), ["start db"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_start_overlapping_a_restart_leaves_one_lifecycle() {
        let journal = Journal::default();
        let sup = slow_stopping_db(&journal);
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        let restart = tokio::spawn({
            let sup = Arc::clone(&sup);
            async move { sup.restart("db", false).await }
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        sup.start("db").await.unwrap();
        restart.await.unwrap().unwrap();
        settle(&sup, all_satisfied).await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Every start but the last is followed by a stop: no lifecycle was left behind.
        assert_eq!(journal.entries(), ["start db", "stop db", "start db"]);
    }

    #[tokio::test(start_paused = true)]
    async fn starting_a_failed_unit_stops_its_remains_first() {
        let journal = Journal::default();
        let db = Arc::new(FakeUnit::service("db", &journal));
        let sup = setup(vec![db.clone()], &[("db", &[])]);
        sup.start_all().await;
        settle(&sup, all_satisfied).await;
        db.crash("killed by OOM");
        settle(&sup, |s| !all_satisfied(s)).await;

        sup.start("db").await.unwrap();
        settle(&sup, all_satisfied).await;

        assert_eq!(journal.entries(), ["start db", "stop db", "start db"]);
    }

    fn graph_of(edges: &[(&str, &[&str])]) -> Graph {
        Graph::new(edges.iter().map(|(unit, deps)| {
            (
                (*unit).to_owned(),
                deps.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>(),
            )
        }))
        .unwrap()
    }

    #[tokio::test(start_paused = true)]
    async fn apply_stops_dropped_units_restarts_changed_ones_and_starts_new_ones() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("backend", &journal)),
                Arc::new(FakeUnit::service("pgadmin", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"]), ("pgadmin", &["db"])],
        );
        sup.set_profile("full");
        sup.start_all().await;
        settle(&sup, all_satisfied).await;
        let before = journal.entries().len();

        let units: Vec<Arc<dyn Unit>> = vec![
            Arc::new(FakeUnit::service("db", &journal)),
            Arc::new(FakeUnit::service("backend", &journal).defined_as("release")),
            Arc::new(FakeUnit::oneshot("build-release", &journal)),
        ];
        let change = sup
            .apply(
                "prod-like",
                units,
                graph_of(&[
                    ("db", &[]),
                    ("build-release", &[]),
                    ("backend", &["db", "build-release"]),
                ]),
            )
            .await
            .unwrap();
        settle(&sup, all_satisfied).await;

        assert_eq!(change.profile, "prod-like");
        assert_eq!(change.stopped, ["pgadmin"]);
        assert_eq!(change.restarted, ["backend"]);
        assert_eq!(change.started, ["build-release"]);
        let after: Vec<String> = journal.entries()[before..].to_vec();
        assert!(!after.contains(&"stop db".to_owned()), "{after:?}");
        assert!(!after.contains(&"start db".to_owned()), "{after:?}");
        let mut names: Vec<String> = sup.status().into_iter().map(|s| s.name).collect();
        names.sort();
        assert_eq!(names, ["backend", "build-release", "db"]);
        assert_eq!(sup.profile(), "prod-like");
    }

    #[tokio::test(start_paused = true)]
    async fn profile_changes_are_announced() {
        let journal = Journal::default();
        let sup = setup(
            vec![Arc::new(FakeUnit::service("db", &journal))],
            &[("db", &[])],
        );
        let mut profiles = sup.subscribe_profile();
        sup.apply(
            "minimal",
            vec![Arc::new(FakeUnit::service("db", &journal)) as Arc<dyn Unit>],
            graph_of(&[("db", &[])]),
        )
        .await
        .unwrap();
        assert_eq!(profiles.recv().await.unwrap(), "minimal");
    }

    #[tokio::test(start_paused = true)]
    async fn a_start_racing_a_switch_that_drops_the_unit_starts_nothing() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("pgadmin", &journal).stop_after(Duration::from_secs(5))),
            ],
            &[("db", &[]), ("pgadmin", &[])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        let switch = tokio::spawn({
            let sup = Arc::clone(&sup);
            let journal = journal.clone();
            async move {
                let units: Vec<Arc<dyn Unit>> = vec![Arc::new(FakeUnit::service("db", &journal))];
                sup.apply("minimal", units, graph_of(&[("db", &[])])).await
            }
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        sup.start("pgadmin").await.unwrap();
        switch.await.unwrap().unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;

        assert_eq!(
            &journal.entries()[2..],
            ["stop pgadmin"],
            "the dropped unit must not be revived"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn apply_restarts_a_kept_unit_whose_dependencies_changed() {
        let journal = Journal::default();
        let sup = setup(
            vec![
                Arc::new(FakeUnit::service("db", &journal)),
                Arc::new(FakeUnit::service("backend", &journal)),
            ],
            &[("db", &[]), ("backend", &["db"])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;
        let before = journal.entries().len();

        let units: Vec<Arc<dyn Unit>> = vec![
            Arc::new(FakeUnit::service("db", &journal)),
            Arc::new(FakeUnit::service("backend", &journal)),
            Arc::new(
                FakeUnit::oneshot("build-release", &journal).ready_after(Duration::from_secs(5)),
            ),
        ];
        let change = sup
            .apply(
                "prod-like",
                units,
                graph_of(&[
                    ("db", &[]),
                    ("build-release", &[]),
                    ("backend", &["db", "build-release"]),
                ]),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert_eq!(
            state(&sup, "backend"),
            UnitState::Waiting {
                on: vec!["build-release".into()]
            }
        );
        settle(&sup, all_satisfied).await;

        assert_eq!(change.restarted, ["backend"]);
        assert_eq!(change.started, ["build-release"]);
        assert!(change.stopped.is_empty());
        assert_eq!(
            &journal.entries()[before..],
            ["stop backend", "start build-release", "start backend"]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_stop_racing_a_switch_that_replaces_the_unit_stops_the_successor() {
        let journal = Journal::default();
        let sup = setup(
            vec![Arc::new(
                FakeUnit::service("backend", &journal).stop_after(Duration::from_secs(5)),
            )],
            &[("backend", &[])],
        );
        sup.start_all().await;
        settle(&sup, all_satisfied).await;

        let successor = Journal::default();
        let switch = tokio::spawn({
            let sup = Arc::clone(&sup);
            let successor = successor.clone();
            async move {
                let units: Vec<Arc<dyn Unit>> = vec![Arc::new(
                    FakeUnit::service("backend", &successor).defined_as("release"),
                )];
                sup.apply("prod-like", units, graph_of(&[("backend", &[])]))
                    .await
            }
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        sup.stop("backend").await.unwrap();
        switch.await.unwrap().unwrap();
        tokio::time::sleep(Duration::from_secs(60)).await;

        assert_eq!(journal.entries(), ["start backend", "stop backend"]);
        assert_eq!(
            successor.entries().last().map(String::as_str),
            Some("stop backend")
        );
        assert_eq!(state(&sup, "backend"), UnitState::Stopped);
    }

    #[tokio::test(start_paused = true)]
    async fn apply_refuses_while_shutting_down() {
        let journal = Journal::default();
        let sup = setup(
            vec![Arc::new(FakeUnit::service("db", &journal))],
            &[("db", &[])],
        );
        sup.stop_all().await;
        let result = sup.apply("x", Vec::new(), graph_of(&[])).await;
        assert_eq!(result, Err(SupervisorError::ShuttingDown));
    }
}
