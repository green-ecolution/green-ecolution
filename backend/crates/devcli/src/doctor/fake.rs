use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;

use super::{Check, Cost, Env, Finding, Trigger};
use crate::{config::StaticVars, sync::lock, units::compose::ComposeCli};

/// A check whose outcome a test sets and changes; counts how often it ran.
pub(crate) struct FakeCheck {
    id: &'static str,
    cost: Cost,
    outcome: Mutex<Result<Vec<Finding>, String>>,
    on_log: Option<&'static str>,
    on_failure: Option<&'static str>,
    runs: AtomicUsize,
}

impl FakeCheck {
    pub(crate) fn new(id: &'static str) -> Self {
        Self {
            id,
            cost: Cost::Cheap,
            outcome: Mutex::new(Ok(Vec::new())),
            on_log: None,
            on_failure: None,
            runs: AtomicUsize::new(0),
        }
    }

    pub(crate) fn deep(mut self) -> Self {
        self.cost = Cost::Deep;
        self
    }

    pub(crate) fn finds(self, findings: Vec<Finding>) -> Self {
        self.set(Ok(findings));
        self
    }

    pub(crate) fn fails(self, reason: &str) -> Self {
        self.set(Err(reason.to_owned()));
        self
    }

    pub(crate) fn on_log(mut self, pattern: &'static str) -> Self {
        self.on_log = Some(pattern);
        self
    }

    pub(crate) fn on_failure(mut self, unit: &'static str) -> Self {
        self.on_failure = Some(unit);
        self
    }

    pub(crate) fn set(&self, outcome: Result<Vec<Finding>, String>) {
        *lock(&self.outcome) = outcome;
    }

    pub(crate) fn runs(&self) -> usize {
        self.runs.load(Ordering::SeqCst)
    }

    pub(crate) fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait]
impl Check for FakeCheck {
    fn id(&self) -> &'static str {
        self.id
    }

    fn cost(&self) -> Cost {
        self.cost
    }

    async fn diagnose(&self, _env: &Env) -> Result<Vec<Finding>, String> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        lock(&self.outcome).clone()
    }

    fn reacts_to(&self, trigger: &Trigger<'_>) -> bool {
        match trigger {
            Trigger::Log(record) => self.on_log.is_some_and(|p| record.message.contains(p)),
            Trigger::Failed { unit } => self.on_failure == Some(*unit),
        }
    }
}

pub(crate) fn test_env(root: &Path) -> Env {
    Env {
        root: root.to_path_buf(),
        compose: ComposeCli::new(root.to_path_buf()),
        vars: Arc::new(StaticVars(HashMap::new())),
        units: Vec::new(),
    }
}
