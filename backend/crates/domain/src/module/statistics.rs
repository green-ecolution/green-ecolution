use async_trait::async_trait;

use crate::RepositoryError;

/// `kind` is a stable discriminator the client labels; the core adds the module key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCount {
    pub kind: &'static str,
    pub count: i64,
}

#[async_trait]
pub trait ModuleStatistics: Send + Sync {
    async fn counts(&self) -> Result<Vec<ModuleCount>, RepositoryError>;
}
