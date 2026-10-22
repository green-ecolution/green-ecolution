use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use futures::future::join_all;
use tokio::{
    sync::{Mutex as AsyncMutex, broadcast, broadcast::error::RecvError},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::{Check, Cost, Env, Finding, Report, Skipped, Trigger, sort};
use crate::{
    supervisor::{Supervisor, SupervisorError, UnitState},
    sync::lock,
    tasks::{TaskError, Tasks},
};

/// The pseudo-unit a fix's output is logged under.
pub const FIX_UNIT: &str = "fix";

/// A crash loop would otherwise run the same check on every restart.
const TRIGGER_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DoctorError {
    #[error("unknown finding `{0}`")]
    UnknownFinding(String),
    #[error("finding `{0}` has no automatic fix")]
    NoFix(String),
    #[error("`{0}` is destructive and needs confirmation")]
    NeedsConfirmation(String),
    #[error("another fix is still running")]
    FixRunning,
    #[error("the fix failed: {0}")]
    FixFailed(String),
}

pub struct Doctor {
    checks: Vec<Arc<dyn Check>>,
    env: Env,
    supervisor: Arc<Supervisor>,
    tasks: Arc<Tasks>,
    state: Mutex<State>,
    changes: broadcast::Sender<Vec<Finding>>,
    fixing: AsyncMutex<()>,
}

#[derive(Default)]
struct State {
    findings: BTreeMap<String, Vec<Finding>>,
    skipped: BTreeMap<String, String>,
    last_triggered: HashMap<String, Instant>,
}

impl Doctor {
    pub fn new(
        checks: Vec<Arc<dyn Check>>,
        env: Env,
        supervisor: Arc<Supervisor>,
        tasks: Arc<Tasks>,
    ) -> Arc<Self> {
        let (changes, _) = broadcast::channel(64);
        Arc::new(Self {
            checks,
            env,
            supervisor,
            tasks,
            state: Mutex::default(),
            changes,
            fixing: AsyncMutex::new(()),
        })
    }

    pub fn findings(&self) -> Vec<Finding> {
        let mut all: Vec<Finding> = lock(&self.state)
            .findings
            .values()
            .flatten()
            .cloned()
            .collect();
        sort(&mut all);
        all
    }

    pub fn report(&self) -> Report {
        self.report_for(true)
    }

    /// Without `deep`, stored results of deep checks stay out: a cheap run must not fail on
    /// what an earlier deep run found.
    fn report_for(&self, deep: bool) -> Report {
        let included = |check: &str| {
            deep || self
                .checks
                .iter()
                .all(|c| c.id() != check || c.cost() == Cost::Cheap)
        };
        let state = lock(&self.state);
        let mut findings: Vec<Finding> = state
            .findings
            .iter()
            .filter(|(check, _)| included(check))
            .flat_map(|(_, findings)| findings.iter().cloned())
            .collect();
        sort(&mut findings);
        let skipped = state
            .skipped
            .iter()
            .filter(|(check, _)| included(check))
            .map(|(check, reason)| Skipped {
                check: check.clone(),
                reason: reason.clone(),
            })
            .collect();
        Report { findings, skipped }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<Finding>> {
        self.changes.subscribe()
    }

    pub async fn run(&self, deep: bool) -> Report {
        let checks: Vec<Arc<dyn Check>> = self
            .checks
            .iter()
            .filter(|check| deep || check.cost() == Cost::Cheap)
            .cloned()
            .collect();
        self.run_checks(&checks, None).await;
        self.report_for(deep)
    }

    async fn run_checks(&self, checks: &[Arc<dyn Check>], unit: Option<&str>) {
        let env = Env {
            units: self.supervisor.status(),
            ..self.env.clone()
        };
        let outcomes = join_all(checks.iter().map(|check| check.diagnose(&env))).await;
        {
            let mut state = lock(&self.state);
            for (check, outcome) in checks.iter().zip(outcomes) {
                let id = check.id().to_owned();
                match outcome {
                    Ok(mut findings) => {
                        if let Some(unit) = unit {
                            for finding in &mut findings {
                                finding.unit.get_or_insert_with(|| unit.to_owned());
                            }
                        }
                        inherit_units(&mut findings, state.findings.get(&id));
                        state.skipped.remove(&id);
                        if findings.is_empty() {
                            state.findings.remove(&id);
                        } else {
                            state.findings.insert(id, findings);
                        }
                    }
                    Err(reason) => {
                        state.skipped.insert(id, reason);
                    }
                }
            }
        }
        let _ = self.changes.send(self.findings());
    }

    pub async fn fix(&self, id: &str, confirmed: bool) -> Result<(), DoctorError> {
        let findings = self.findings();
        let finding = findings
            .iter()
            .find(|finding| finding.id == id)
            .ok_or_else(|| DoctorError::UnknownFinding(id.to_owned()))?;
        let fix = finding
            .fix
            .clone()
            .ok_or_else(|| DoctorError::NoFix(id.to_owned()))?;
        let affected: Vec<&str> = findings
            .iter()
            .filter(|other| {
                other
                    .fix
                    .as_ref()
                    .is_some_and(|other| other.command == fix.command)
            })
            .map(|other| other.check.as_str())
            .collect();
        let spec = self
            .tasks
            .command(FIX_UNIT, &fix.command, Path::new(&fix.cwd), fix.destructive)
            .map_err(|err| DoctorError::FixFailed(err.to_string()))?;
        if spec.destructive && !confirmed {
            return Err(DoctorError::NeedsConfirmation(fix.command));
        }
        let _running = self
            .fixing
            .try_lock()
            .map_err(|_| DoctorError::FixRunning)?;
        self.tasks.run(spec, true).await.map_err(|err| match err {
            TaskError::Supervisor(SupervisorError::TaskRunning(_)) => DoctorError::FixRunning,
            TaskError::Failed { reason, .. } => DoctorError::FixFailed(reason),
            other => DoctorError::FixFailed(other.to_string()),
        })?;
        let checks: Vec<Arc<dyn Check>> = self
            .checks
            .iter()
            .filter(|check| affected.contains(&check.id()))
            .cloned()
            .collect();
        self.run_checks(&checks, finding.unit.as_deref()).await;
        Ok(())
    }

    pub fn watch(self: &Arc<Self>, shutdown: CancellationToken) -> JoinHandle<()> {
        let doctor = Arc::clone(self);
        tokio::spawn(async move {
            let mut logs = doctor.supervisor.logs().subscribe();
            let mut states = doctor.supervisor.subscribe();
            let mut profiles = doctor.supervisor.subscribe_profile();
            loop {
                let (checks, unit) = tokio::select! {
                    () = shutdown.cancelled() => return,
                    // A switch changes which units run, so the start-up findings may no longer hold.
                    received = profiles.recv() => match received {
                        Ok(_) | Err(RecvError::Lagged(_)) => {
                            let doctor = Arc::clone(&doctor);
                            tokio::spawn(async move { doctor.run(false).await });
                            continue;
                        }
                        Err(RecvError::Closed) => return,
                    },
                    received = logs.recv() => match received {
                        Ok(record) if record.unit != FIX_UNIT => {
                            (doctor.triggered(&Trigger::Log(&record)), Some(record.unit))
                        }
                        Ok(_) | Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => return,
                    },
                    received = states.recv() => match received {
                        Ok(status) => match status.state {
                            UnitState::Failed { .. } => (
                                doctor.triggered(&Trigger::Failed { unit: &status.name }),
                                Some(status.name),
                            ),
                            // The settled unit did not cause these findings.
                            UnitState::Ready | UnitState::Done => (doctor.settled(&status.name), None),
                            _ => continue,
                        },
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => return,
                    },
                };
                if checks.is_empty() {
                    continue;
                }
                let doctor = Arc::clone(&doctor);
                tokio::spawn(async move { doctor.run_checks(&checks, unit.as_deref()).await });
            }
        })
    }

    /// Checks reacting to `trigger` that have not run for a trigger within the cooldown.
    fn triggered(&self, trigger: &Trigger<'_>) -> Vec<Arc<dyn Check>> {
        let now = Instant::now();
        let mut state = lock(&self.state);
        self.checks
            .iter()
            .filter(|check| check.reacts_to(trigger))
            .filter(|check| {
                let recent = state
                    .last_triggered
                    .get(check.id())
                    .is_some_and(|at| now.duration_since(*at) < TRIGGER_COOLDOWN);
                if !recent {
                    state.last_triggered.insert(check.id().to_owned(), now);
                }
                !recent
            })
            .cloned()
            .collect()
    }

    /// Cheap checks holding findings, to run again once a unit is ready or done: the start-up
    /// run may have raced the units. Cooled down per unit and apart from triggers, so none of them starves another.
    fn settled(&self, unit: &str) -> Vec<Arc<dyn Check>> {
        let now = Instant::now();
        let mut state = lock(&self.state);
        self.checks
            .iter()
            .filter(|check| check.cost() == Cost::Cheap)
            .filter(|check| {
                if !state.findings.contains_key(check.id()) {
                    return false;
                }
                let key = format!("{}@settled:{unit}", check.id());
                let recent = state
                    .last_triggered
                    .get(&key)
                    .is_some_and(|at| now.duration_since(*at) < TRIGGER_COOLDOWN);
                if !recent {
                    state.last_triggered.insert(key, now);
                }
                !recent
            })
            .cloned()
            .collect()
    }
}

/// A run without a trigger must not drop the unit an earlier triggered run attributed.
fn inherit_units(fresh: &mut [Finding], previous: Option<&Vec<Finding>>) {
    let Some(previous) = previous else {
        return;
    };
    for finding in fresh.iter_mut().filter(|finding| finding.unit.is_none()) {
        finding.unit = previous
            .iter()
            .find(|old| old.id == finding.id)
            .and_then(|old| old.unit.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use chrono::Utc;

    use super::*;
    use crate::{
        config::LogFormat,
        doctor::{
            Fix, Severity,
            fake::{FakeCheck, test_env},
        },
        graph::Graph,
        logs::{Filter, LogStore, parse_line},
        state::ProcessRegistry,
        supervisor::Supervisor,
        units::{
            Origin, Unit,
            fake::{FakeUnit, Journal},
        },
    };

    struct Fixture {
        _dir: tempfile::TempDir,
        supervisor: Arc<Supervisor>,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("geco.toml"),
            "[tasks]\ndestructive = [\"seed-reset\"]\n",
        )
        .unwrap();
        let units: Vec<Arc<dyn Unit>> =
            vec![Arc::new(FakeUnit::service("db", &Journal::default()))];
        let graph = Graph::new([("db".to_owned(), Vec::<String>::new())]).unwrap();
        Fixture {
            supervisor: Supervisor::new(units, graph, LogStore::new(100)),
            _dir: dir,
        }
    }

    fn doctor(f: &Fixture, checks: Vec<Arc<dyn Check>>) -> Arc<Doctor> {
        let tasks = Arc::new(Tasks::new(
            f._dir.path().to_path_buf(),
            ProcessRegistry::in_memory(),
            Arc::clone(&f.supervisor),
        ));
        Doctor::new(
            checks,
            test_env(f._dir.path()),
            Arc::clone(&f.supervisor),
            tasks,
        )
    }

    fn with_fix(id: &str, command: &str) -> Finding {
        Finding::new(id, Severity::Error, format!("{id} is off")).fix(Fix {
            command: command.into(),
            cwd: String::new(),
            destructive: false,
        })
    }

    fn fix_logs(f: &Fixture) -> Vec<String> {
        f.supervisor
            .logs()
            .snapshot(&[FIX_UNIT.to_owned()], &Filter::default(), 100, Utc::now())
            .into_iter()
            .map(|r| r.message)
            .collect()
    }

    #[tokio::test]
    async fn runs_keep_findings_per_check_and_broadcast_changes() {
        let f = fixture();
        let flaky = FakeCheck::new("docker")
            .finds(vec![with_fix("docker", "true")])
            .arc();
        let doc = doctor(&f, vec![flaky.clone(), FakeCheck::new("deep").deep().arc()]);
        let mut changes = doc.subscribe();

        let report = doc.run(false).await;
        assert_eq!(report.findings.len(), 1);
        assert_eq!(changes.recv().await.unwrap().len(), 1);

        flaky.set(Err("Docker is not reachable".into()));
        let report = doc.run(false).await;
        assert_eq!(
            report.findings.len(),
            1,
            "a check that could not run keeps its findings"
        );
        assert_eq!(report.skipped[0].check, "docker");
    }

    #[tokio::test]
    async fn cheap_runs_leave_out_the_stored_deep_results() {
        let f = fixture();
        let deep = FakeCheck::new("sqlx-cache-prepare")
            .deep()
            .finds(vec![with_fix("sqlx-cache-prepare", "true")])
            .arc();
        let doc = doctor(&f, vec![deep, FakeCheck::new("cheap").arc()]);

        assert!(doc.run(true).await.has_errors());
        let cheap = doc.run(false).await;

        assert!(cheap.findings.is_empty());
        assert!(!cheap.has_errors());
        assert_eq!(
            doc.findings().len(),
            1,
            "the TUI keeps the last deep result"
        );
        assert_eq!(doc.report().findings.len(), 1);
    }

    #[tokio::test]
    async fn a_fix_checks_again_every_check_that_shares_its_command() {
        let f = fixture();
        let behind = FakeCheck::new("dev-db-migrations")
            .finds(vec![with_fix("dev-db-migrations", "echo migrated")])
            .arc();
        let binary = FakeCheck::new("migrate-binary")
            .finds(vec![with_fix("migrate-binary", "echo migrated")])
            .arc();
        let other = FakeCheck::new("ports")
            .finds(vec![with_fix("ports", "echo other")])
            .arc();
        let doc = doctor(&f, vec![behind.clone(), binary.clone(), other.clone()]);
        doc.run(false).await;

        behind.set(Ok(Vec::new()));
        binary.set(Ok(Vec::new()));
        doc.fix("dev-db-migrations", false).await.unwrap();

        let ids: Vec<String> = doc.findings().into_iter().map(|f| f.id).collect();
        assert_eq!(ids, ["ports"]);
        assert_eq!(binary.runs(), 2);
        assert_eq!(other.runs(), 1);
    }

    #[tokio::test]
    async fn cheap_runs_leave_out_deep_checks_that_could_not_run() {
        let f = fixture();
        let deep = FakeCheck::new("sqlx-cache-prepare")
            .deep()
            .fails("cargo-sqlx is not installed")
            .arc();
        let doc = doctor(&f, vec![deep]);

        assert_eq!(doc.run(true).await.skipped.len(), 1);
        assert!(doc.run(false).await.skipped.is_empty());
        assert_eq!(doc.report().skipped.len(), 1);
    }

    #[tokio::test]
    async fn a_fix_runs_logs_under_fix_and_checks_again() {
        let f = fixture();
        let check = FakeCheck::new("migrations")
            .finds(vec![with_fix("migrations", "echo fixed")])
            .arc();
        let doc = doctor(&f, vec![check.clone()]);
        doc.run(false).await;

        check.set(Ok(Vec::new()));
        doc.fix("migrations", false).await.unwrap();

        assert_eq!(fix_logs(&f), ["$ echo fixed", "fixed"]);
        assert!(doc.findings().is_empty());
        assert_eq!(check.runs(), 2);
    }

    #[tokio::test]
    async fn fixes_are_refused_when_they_cannot_or_must_not_run() {
        let f = fixture();
        let destructive = Finding::new("testcontainers", Severity::Hint, "leftovers")
            .fix(Fix::just("test-clean").destructive());
        let no_fix = Finding::new("ports", Severity::Error, "taken");
        let failing = with_fix("failing", "exit 3");
        let doc = doctor(
            &f,
            vec![
                FakeCheck::new("testcontainers")
                    .finds(vec![destructive])
                    .arc(),
                FakeCheck::new("ports").finds(vec![no_fix]).arc(),
                FakeCheck::new("failing").finds(vec![failing]).arc(),
            ],
        );
        doc.run(false).await;

        assert_eq!(
            doc.fix("nope", true).await,
            Err(DoctorError::UnknownFinding("nope".into()))
        );
        assert_eq!(
            doc.fix("ports", true).await,
            Err(DoctorError::NoFix("ports".into()))
        );
        assert_eq!(
            doc.fix("testcontainers", false).await,
            Err(DoctorError::NeedsConfirmation("just test-clean".into()))
        );
        assert!(matches!(
            doc.fix("failing", false).await,
            Err(DoctorError::FixFailed(reason)) if reason.contains("exit status: 3")
        ));
    }

    #[tokio::test]
    async fn a_fix_running_a_listed_recipe_needs_confirmation() {
        let f = fixture();
        let check = FakeCheck::new("seed")
            .finds(vec![
                Finding::new("seed", Severity::Hint, "stale").fix(Fix::just("seed-reset")),
            ])
            .arc();
        let doc = doctor(&f, vec![check]);
        doc.run(false).await;

        assert_eq!(
            doc.fix("seed", false).await,
            Err(DoctorError::NeedsConfirmation("just seed-reset".into()))
        );
    }

    #[tokio::test]
    async fn a_running_fix_is_listed_as_a_task() {
        let f = fixture();
        let check = FakeCheck::new("slow")
            .finds(vec![with_fix("slow", "sleep 1")])
            .arc();
        let doc = doctor(&f, vec![check]);
        doc.run(false).await;
        let mut events = f.supervisor.subscribe();

        let fix = tokio::spawn({
            let doc = Arc::clone(&doc);
            async move { doc.fix("slow", false).await }
        });
        let starting = events.recv().await.unwrap();
        assert_eq!(
            (starting.name.as_str(), starting.origin),
            (FIX_UNIT, Origin::Task)
        );
        fix.await.unwrap().unwrap();
        assert!(f.supervisor.status().iter().all(|s| s.name != FIX_UNIT));
    }

    #[tokio::test]
    async fn later_runs_keep_the_unit_a_trigger_attributed() {
        let f = fixture();
        let behind = Finding::new("dev-db-migrations", Severity::Error, "2 behind");
        let migrations = FakeCheck::new("dev-db-migrations")
            .finds(vec![behind.clone()])
            .arc();
        let check: Arc<dyn Check> = migrations.clone();
        let doc = doctor(&f, vec![check.clone()]);

        doc.run_checks(&[check], Some("migrations")).await;
        doc.run(false).await;
        assert_eq!(doc.findings()[0].unit.as_deref(), Some("migrations"));

        migrations.set(Ok(Vec::new()));
        doc.run(false).await;
        migrations.set(Ok(vec![behind]));
        doc.run(false).await;
        assert_eq!(
            doc.findings()[0].unit,
            None,
            "a reappearing finding starts unattributed"
        );
    }

    #[tokio::test]
    async fn a_settled_unit_rechecks_the_checks_holding_findings() {
        let dir = tempfile::tempdir().unwrap();
        let units: Vec<Arc<dyn Unit>> = vec![Arc::new(FakeUnit::oneshot(
            "migrations",
            &Journal::default(),
        ))];
        let graph = Graph::new([("migrations".to_owned(), Vec::<String>::new())]).unwrap();
        let f = Fixture {
            supervisor: Supervisor::new(units, graph, LogStore::new(100)),
            _dir: dir,
        };
        let migrations = FakeCheck::new("dev-db-migrations")
            .finds(vec![Finding::new(
                "dev-db-migrations",
                Severity::Error,
                "2 behind",
            )])
            .on_failure("migrations")
            .arc();
        let doc = doctor(&f, vec![migrations.clone()]);
        doc.run(false).await;
        assert!(
            !doc.triggered(&Trigger::Failed { unit: "migrations" })
                .is_empty(),
            "a trigger shortly before must not suppress the settled re-check"
        );
        migrations.set(Ok(Vec::new()));

        let shutdown = CancellationToken::new();
        let watcher = doc.watch(shutdown.clone());
        let mut changes = doc.subscribe();
        tokio::time::sleep(Duration::from_millis(50)).await;
        f.supervisor.start("migrations").await.unwrap();

        let findings = tokio::time::timeout(Duration::from_secs(5), changes.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(findings.is_empty());
        assert!(doc.findings().is_empty());
        shutdown.cancel();
        watcher.await.unwrap();
    }

    #[tokio::test]
    async fn a_profile_switch_runs_the_cheap_checks_again() {
        let f = fixture();
        let ports = FakeCheck::new("ports")
            .finds(vec![Finding::new("ports", Severity::Error, "taken")])
            .arc();
        let doc = doctor(&f, vec![ports.clone()]);
        doc.run(false).await;
        assert_eq!(doc.findings().len(), 1);
        ports.set(Ok(vec![]));

        let shutdown = CancellationToken::new();
        let watcher = doc.watch(shutdown.clone());
        let mut changes = doc.subscribe();
        tokio::time::sleep(Duration::from_millis(50)).await;
        f.supervisor.set_profile("x");

        let findings = tokio::time::timeout(Duration::from_secs(5), changes.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(findings.is_empty());
        assert!(doc.findings().is_empty());
        shutdown.cancel();
        watcher.await.unwrap();
    }

    #[tokio::test]
    async fn settling_rechecks_only_cheap_checks_with_findings_once_per_cooldown() {
        let f = fixture();
        let found = Finding::new("ports", Severity::Error, "taken");
        let deep_found = Finding::new("deep", Severity::Error, "stale");
        let doc = doctor(
            &f,
            vec![
                FakeCheck::new("ports").finds(vec![found]).arc(),
                FakeCheck::new("clean").arc(),
                FakeCheck::new("deep").deep().finds(vec![deep_found]).arc(),
            ],
        );
        doc.run(true).await;

        let ids: Vec<&str> = doc.settled("db").iter().map(|check| check.id()).collect();
        assert_eq!(ids, ["ports"]);
        assert!(
            doc.settled("db").is_empty(),
            "the cooldown holds the second one"
        );
    }

    #[tokio::test]
    async fn every_settling_unit_gets_its_own_recheck() {
        let f = fixture();
        let check = FakeCheck::new("dev-db-migrations")
            .finds(vec![Finding::new(
                "dev-db-migrations",
                Severity::Error,
                "2 behind",
            )])
            .arc();
        let doc = doctor(&f, vec![check]);
        doc.run(false).await;

        assert_eq!(doc.settled("db").len(), 1);
        assert_eq!(
            doc.settled("migrations").len(),
            1,
            "another unit settling right after must not be held by the first"
        );
        assert!(doc.settled("migrations").is_empty());
    }

    #[tokio::test]
    async fn log_lines_and_failures_trigger_checks_once_per_cooldown() {
        let f = fixture();
        let migrations = FakeCheck::new("dev-db-migrations")
            .finds(vec![Finding::new(
                "dev-db-migrations",
                Severity::Error,
                "2 behind",
            )])
            .on_log("does not exist")
            .on_failure("migrations")
            .arc();
        let doc = doctor(&f, vec![migrations.clone()]);
        let shutdown = CancellationToken::new();
        let watcher = doc.watch(shutdown.clone());
        let mut changes = doc.subscribe();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let record = |message: &str| parse_line("backend", LogFormat::Plain, message, Utc::now());
        f.supervisor
            .logs()
            .push(record(r#"column "x" does not exist"#));
        let findings = tokio::time::timeout(Duration::from_secs(5), changes.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(findings[0].unit.as_deref(), Some("backend"));

        f.supervisor
            .logs()
            .push(record(r#"column "y" does not exist"#));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(
            migrations.runs(),
            1,
            "the cooldown suppresses the second trigger"
        );

        assert!(
            doc.triggered(&Trigger::Failed { unit: "migrations" })
                .is_empty()
        );
        shutdown.cancel();
        watcher.await.unwrap();
    }
}
