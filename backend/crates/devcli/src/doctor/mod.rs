pub mod checks;
#[cfg(test)]
pub(crate) mod fake;
pub mod service;

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    config::VarSource,
    logs::LogRecord,
    supervisor::{UnitState, UnitStatus},
    units::compose::ComposeCli,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Hint,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cost {
    Cheap,
    Deep,
}

/// Shown to the user before it runs, so it never carries a secret; recipes resolve those.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    pub command: String,
    /// Relative to the repository root; empty for the root itself.
    pub cwd: String,
    pub destructive: bool,
}

impl Fix {
    pub fn just(recipe: &str) -> Self {
        Self {
            command: format!("just {recipe}"),
            cwd: String::new(),
            destructive: false,
        }
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// `<check>` or `<check>:<subject>`; stable across runs so a fix can refer to it.
    pub id: String,
    pub check: String,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    /// The unit whose failure or log lines led to this finding.
    pub unit: Option<String>,
    pub fix: Option<Fix>,
}

impl Finding {
    pub fn new(check: &str, severity: Severity, title: impl Into<String>) -> Self {
        Self {
            id: check.to_owned(),
            check: check.to_owned(),
            severity,
            title: title.into(),
            detail: String::new(),
            unit: None,
            fix: None,
        }
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.id = format!("{}:{subject}", self.check);
        self
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_owned());
        self
    }

    pub fn fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skipped {
    pub check: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Checks that could not run, e.g. because Docker is down; not the same as "all good".
    pub skipped: Vec<Skipped>,
}

impl Report {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }
}

pub enum Trigger<'a> {
    Log(&'a LogRecord),
    Failed { unit: &'a str },
}

#[derive(Clone)]
pub struct Env {
    pub root: PathBuf,
    pub compose: ComposeCli,
    pub vars: Arc<dyn VarSource + Send + Sync>,
    /// The supervisor's units; empty when no supervisor runs.
    pub units: Vec<UnitStatus>,
}

impl Env {
    /// Whether the supervisor runs `unit` or is bringing it up.
    pub fn runs(&self, unit: &str) -> bool {
        self.units.iter().any(|status| {
            status.name == unit
                && matches!(
                    status.state,
                    UnitState::Waiting { .. } | UnitState::Starting | UnitState::Ready
                )
        })
    }
}

#[async_trait]
pub trait Check: Send + Sync {
    fn id(&self) -> &'static str;

    fn cost(&self) -> Cost {
        Cost::Cheap
    }

    /// `Err` means the check could not run, not that something is wrong.
    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String>;

    /// Log lines or unit failures after which this check runs again on its own.
    fn reacts_to(&self, _trigger: &Trigger<'_>) -> bool {
        false
    }
}

pub async fn run(checks: &[Arc<dyn Check>], env: &Env, deep: bool) -> Report {
    let selected: Vec<&Arc<dyn Check>> = checks
        .iter()
        .filter(|check| deep || check.cost() == Cost::Cheap)
        .collect();
    let outcomes =
        futures::future::join_all(selected.iter().map(|check| check.diagnose(env))).await;
    let mut report = Report::default();
    for (check, outcome) in selected.into_iter().zip(outcomes) {
        match outcome {
            Ok(findings) => report.findings.extend(findings),
            Err(reason) => report.skipped.push(Skipped {
                check: check.id().to_owned(),
                reason,
            }),
        }
    }
    sort(&mut report.findings);
    report
}

pub fn sort(findings: &mut [Finding]) {
    findings.sort_by(|a, b| b.severity.cmp(&a.severity).then_with(|| a.id.cmp(&b.id)));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;

    use super::{
        fake::{FakeCheck, test_env},
        *,
    };
    use crate::{
        supervisor::{UnitState, UnitStatus},
        units::{Origin, UnitKind},
    };

    fn finding(check: &str, severity: Severity) -> Finding {
        Finding::new(check, severity, format!("{check} is off"))
    }

    #[tokio::test]
    async fn run_collects_findings_errors_first_and_lists_checks_that_could_not_run() {
        let dir = tempfile::tempdir().unwrap();
        let checks: Vec<Arc<dyn Check>> = vec![
            FakeCheck::new("b-hint")
                .finds(vec![finding("b-hint", Severity::Hint)])
                .arc(),
            FakeCheck::new("c-error")
                .finds(vec![finding("c-error", Severity::Error)])
                .arc(),
            FakeCheck::new("a-hint")
                .finds(vec![finding("a-hint", Severity::Hint)])
                .arc(),
            FakeCheck::new("docker")
                .fails("Docker is not reachable")
                .arc(),
        ];

        let report = run(&checks, &test_env(dir.path()), false).await;

        let ids: Vec<&str> = report.findings.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids, ["c-error", "a-hint", "b-hint"]);
        assert_eq!(
            report.skipped,
            [Skipped {
                check: "docker".into(),
                reason: "Docker is not reachable".into()
            }]
        );
        assert!(report.has_errors());
    }

    #[tokio::test]
    async fn deep_checks_run_only_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let deep = FakeCheck::new("deep").deep().arc();
        let checks: Vec<Arc<dyn Check>> = vec![deep.clone(), FakeCheck::new("cheap").arc()];

        run(&checks, &test_env(dir.path()), false).await;
        assert_eq!(deep.runs(), 0);
        run(&checks, &test_env(dir.path()), true).await;
        assert_eq!(deep.runs(), 1);
    }

    #[test]
    fn findings_name_their_subject_and_carry_a_fix() {
        let f = Finding::new("package-dist", Severity::Hint, "ui is stale")
            .subject("ui")
            .unit("frontend")
            .detail("why")
            .fix(Fix::just("test-clean").destructive());

        assert_eq!(f.id, "package-dist:ui");
        assert_eq!(f.check, "package-dist");
        assert_eq!(f.unit.as_deref(), Some("frontend"));
        assert_eq!(
            f.fix,
            Some(Fix {
                command: "just test-clean".into(),
                cwd: String::new(),
                destructive: true
            })
        );
        assert!(!Report::default().has_errors());
    }

    #[test]
    fn env_knows_which_units_the_supervisor_runs() {
        let dir = tempfile::tempdir().unwrap();
        let status = |name: &str, state| UnitStatus {
            name: name.into(),
            kind: UnitKind::Service,
            origin: Origin::Local,
            state,
            since: Utc::now(),
        };
        let mut env = test_env(dir.path());
        env.units = vec![
            status("backend", UnitState::Ready),
            status("frontend", UnitState::Failed { reason: "x".into() }),
            status("db", UnitState::Waiting { on: vec![] }),
        ];

        assert!(env.runs("backend"));
        assert!(env.runs("db"));
        assert!(!env.runs("frontend"));
        assert!(!env.runs("unknown"));
    }

    #[test]
    fn findings_serialize_for_json_output() {
        let json = serde_json::to_value(finding("ports", Severity::Error)).unwrap();
        assert_eq!(json["severity"], "error");
        assert_eq!(json["id"], "ports");
    }
}
