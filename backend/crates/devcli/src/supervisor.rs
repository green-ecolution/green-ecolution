use std::{
    collections::BTreeMap,
    fmt,
    sync::{
        Arc, Mutex,
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
    sync::lock,
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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SupervisorError {
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    #[error("the supervisor is shutting down")]
    ShuttingDown,
}

pub struct Supervisor {
    entries: BTreeMap<String, Entry>,
    graph: Graph,
    logs: Arc<LogStore>,
    events: broadcast::Sender<UnitStatus>,
    /// Set first thing in `stop_all`; no lifecycle is spawned afterwards.
    closing: AtomicBool,
}

struct Entry {
    unit: Arc<dyn Unit>,
    status: Mutex<UnitStatus>,
    task: Mutex<Option<JoinHandle<()>>>,
    /// Held across a whole start/stop/restart so control operations on one unit never overlap.
    control: AsyncMutex<()>,
}

impl Supervisor {
    /// `graph` must contain exactly the given units; `assemble` guarantees that.
    pub fn new(units: Vec<Arc<dyn Unit>>, graph: Graph, logs: Arc<LogStore>) -> Arc<Self> {
        let (events, _) = broadcast::channel(1024);
        let entries = units
            .into_iter()
            .map(|unit| {
                let status = UnitStatus {
                    name: unit.name().to_owned(),
                    kind: unit.kind(),
                    origin: unit.origin(),
                    state: UnitState::Stopped,
                    since: Utc::now(),
                };
                let entry = Entry {
                    unit,
                    status: Mutex::new(status),
                    task: Mutex::new(None),
                    control: AsyncMutex::new(()),
                };
                (entry.unit.name().to_owned(), entry)
            })
            .collect();
        Arc::new(Self {
            entries,
            graph,
            logs,
            events,
            closing: AtomicBool::new(false),
        })
    }

    pub fn logs(&self) -> &Arc<LogStore> {
        &self.logs
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UnitStatus> {
        self.events.subscribe()
    }

    pub fn status(&self) -> Vec<UnitStatus> {
        self.graph
            .start_order()
            .iter()
            .filter_map(|name| self.entries.get(name))
            .map(|entry| lock(&entry.status).clone())
            .collect()
    }

    pub async fn start_all(self: &Arc<Self>) {
        for name in self.graph.start_order() {
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
        self.spawn_lifecycle(name, entry).await
    }

    pub async fn stop(&self, name: &str) -> Result<(), SupervisorError> {
        let entry = self.entry(name)?;
        let _control = entry.control.lock().await;
        self.stop_entry(entry).await;
        Ok(())
    }

    pub async fn stop_all(&self) {
        self.closing.store(true, Ordering::SeqCst);
        for name in self.graph.start_order().iter().rev() {
            if let Some(entry) = self.entries.get(name) {
                let _control = entry.control.lock().await;
                self.stop_entry(entry).await;
            }
        }
    }

    /// Fails with `ShuttingDown` once `stop_all` began; units it already stopped stay stopped.
    pub async fn restart(
        self: &Arc<Self>,
        name: &str,
        cascade: bool,
    ) -> Result<(), SupervisorError> {
        let entry = self.entry(name)?;
        let dependents = if cascade {
            self.graph.transitive_dependents(name)
        } else {
            Vec::new()
        };
        // Taking several control locks only ever in start order rules out deadlocks.
        let mut guards = Vec::with_capacity(dependents.len() + 1);
        let affected = self
            .graph
            .start_order()
            .iter()
            .filter(|unit| *unit == name || dependents.contains(unit))
            .filter_map(|unit| self.entries.get(unit));
        for entry in affected {
            guards.push(entry.control.lock().await);
        }
        if self.is_closing() {
            return Err(SupervisorError::ShuttingDown);
        }
        for dependent in dependents.iter().rev() {
            if let Some(dependent) = self.entries.get(dependent) {
                self.stop_entry(dependent).await;
            }
        }
        self.stop_entry(entry).await;
        self.spawn_lifecycle(name, entry).await?;
        for dependent in &dependents {
            if let Some(entry) = self.entries.get(dependent) {
                self.spawn_lifecycle(dependent, entry).await?;
            }
        }
        Ok(())
    }

    fn entry(&self, name: &str) -> Result<&Entry, SupervisorError> {
        self.entries
            .get(name)
            .ok_or_else(|| SupervisorError::UnknownUnit(name.to_owned()))
    }

    fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    /// The caller holds `entry.control`. Checking `closing` under that lock guarantees that
    /// `stop_all` either sees the new lifecycle or no lifecycle is spawned.
    async fn spawn_lifecycle(
        self: &Arc<Self>,
        name: &str,
        entry: &Entry,
    ) -> Result<(), SupervisorError> {
        if lock(&entry.task).is_some() {
            // A displaced lifecycle may still own a process; stop it instead of orphaning it.
            self.stop_entry(entry).await;
        }
        if self.is_closing() {
            return Err(SupervisorError::ShuttingDown);
        }
        let this = Arc::clone(self);
        let owned = name.to_owned();
        let handle = tokio::spawn(async move { this.lifecycle(&owned).await });
        *lock(&entry.task) = Some(handle);
        Ok(())
    }

    async fn lifecycle(&self, name: &str) {
        let Some(entry) = self.entries.get(name) else {
            return;
        };
        // Subscribe before the first check so no dependency change slips through.
        let mut events = self.events.subscribe();
        loop {
            let pending: Vec<String> = self
                .graph
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
        self.entries
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
}
