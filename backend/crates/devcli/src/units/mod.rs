pub mod probe;
pub mod process;

#[cfg(test)]
pub mod fake;

use std::{fmt, sync::Arc};

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::logs::LogStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitKind {
    Service,
    Oneshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Local,
    Compose,
}

#[derive(Debug, thiserror::Error)]
pub enum UnitError {
    #[error("cannot run `{command}`: {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{0}")]
    Exited(String),
    #[error("{0}")]
    Docker(String),
    #[error("{0}")]
    Verify(String),
    #[error("{0}")]
    Hook(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// `exited` resolves with a reason once a service stops on its own; for oneshots it never does.
pub struct Running {
    pub exited: BoxFuture<'static, String>,
}

impl Running {
    pub fn finished() -> Self {
        Self {
            exited: Box::pin(futures::future::pending()),
        }
    }
}

impl fmt::Debug for Running {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Running")
    }
}

#[async_trait]
pub trait Unit: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> UnitKind;
    fn origin(&self) -> Origin;
    /// Resolves once a service is ready or a oneshot has finished successfully.
    async fn start(&self, logs: Arc<LogStore>) -> Result<Running, UnitError>;
    /// Must also clean up after a `start` that was cancelled half-way.
    async fn stop(&self) -> Result<(), UnitError>;
}
