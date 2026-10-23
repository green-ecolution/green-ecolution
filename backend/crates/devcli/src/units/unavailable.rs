use std::sync::Arc;

use async_trait::async_trait;

use super::{Origin, Running, Unit, UnitError, UnitKind, fingerprint};
use crate::logs::LogStore;

/// Stands in for a compose service while Docker is unreachable, so local units still run and
/// the reason shows up on the unit itself.
pub struct UnavailableUnit {
    name: String,
    reason: String,
}

impl UnavailableUnit {
    pub fn new(name: String, reason: String) -> Self {
        Self { name, reason }
    }
}

#[async_trait]
impl Unit for UnavailableUnit {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> UnitKind {
        UnitKind::Service
    }

    fn origin(&self) -> Origin {
        Origin::Compose
    }

    fn definition(&self) -> u64 {
        fingerprint(("unavailable", &self.name, &self.reason))
    }

    async fn start(&self, _logs: Arc<LogStore>) -> Result<Running, UnitError> {
        Err(UnitError::Docker(self.reason.clone()))
    }

    async fn stop(&self) -> Result<(), UnitError> {
        Ok(())
    }
}
