use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::oneshot;

use super::{Origin, Running, Unit, UnitError, UnitKind, fingerprint};
use crate::logs::LogStore;

#[derive(Clone, Default)]
pub struct Journal(Arc<Mutex<Vec<String>>>);

impl Journal {
    pub fn entries(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }

    fn push(&self, entry: String) {
        self.0.lock().unwrap().push(entry);
    }
}

pub struct FakeUnit {
    name: String,
    kind: UnitKind,
    tag: String,
    ready_after: Duration,
    stop_after: Duration,
    fail_with: Option<String>,
    journal: Journal,
    exit: Mutex<Option<oneshot::Sender<String>>>,
}

impl FakeUnit {
    pub fn service(name: &str, journal: &Journal) -> Self {
        Self::new(name, UnitKind::Service, journal)
    }

    pub fn oneshot(name: &str, journal: &Journal) -> Self {
        Self::new(name, UnitKind::Oneshot, journal)
    }

    fn new(name: &str, kind: UnitKind, journal: &Journal) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            tag: String::new(),
            ready_after: Duration::from_millis(10),
            stop_after: Duration::ZERO,
            fail_with: None,
            journal: journal.clone(),
            exit: Mutex::new(None),
        }
    }

    pub fn defined_as(mut self, tag: &str) -> Self {
        self.tag = tag.to_owned();
        self
    }

    pub fn ready_after(mut self, delay: Duration) -> Self {
        self.ready_after = delay;
        self
    }

    pub fn stop_after(mut self, delay: Duration) -> Self {
        self.stop_after = delay;
        self
    }

    pub fn failing(mut self, reason: &str) -> Self {
        self.fail_with = Some(reason.to_owned());
        self
    }

    pub fn crash(&self, reason: &str) {
        if let Some(tx) = self.exit.lock().unwrap().take() {
            let _ = tx.send(reason.to_owned());
        }
    }
}

#[async_trait]
impl Unit for FakeUnit {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> UnitKind {
        self.kind
    }

    fn origin(&self) -> Origin {
        Origin::Local
    }

    fn definition(&self) -> u64 {
        fingerprint((&self.name, format!("{:?}", self.kind), &self.tag))
    }

    async fn start(&self, _logs: Arc<LogStore>) -> Result<Running, UnitError> {
        self.journal.push(format!("start {}", self.name));
        tokio::time::sleep(self.ready_after).await;
        if let Some(reason) = &self.fail_with {
            return Err(UnitError::Exited(reason.clone()));
        }
        if self.kind == UnitKind::Oneshot {
            return Ok(Running::finished());
        }
        let (tx, rx) = oneshot::channel();
        *self.exit.lock().unwrap() = Some(tx);
        Ok(Running {
            exited: Box::pin(async move { rx.await.unwrap_or_else(|_| "dropped".to_owned()) }),
        })
    }

    async fn stop(&self) -> Result<(), UnitError> {
        self.journal.push(format!("stop {}", self.name));
        tokio::time::sleep(self.stop_after).await;
        self.exit.lock().unwrap().take();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fakes_differ_by_their_tag() {
        let journal = Journal::default();
        let a = FakeUnit::service("db", &journal);
        let b = FakeUnit::service("db", &journal).defined_as("v2");
        assert_ne!(a.definition(), b.definition());
        assert_eq!(
            a.definition(),
            FakeUnit::service("db", &journal).definition()
        );
    }
}
