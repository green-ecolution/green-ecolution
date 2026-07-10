use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    start_point::{StartPoint, StartPointDraft},
};

/// Read-side access to start points.
#[async_trait]
pub trait StartPointReader: Send + Sync {
    async fn all(&self) -> Result<Vec<StartPoint>, RepositoryError>;
    async fn by_id(&self, id: Id<StartPoint>) -> Result<StartPoint, RepositoryError>;
}

/// Write-side access to start points.
#[async_trait]
pub trait StartPointWriter: Send + Sync {
    async fn save_new(&self, draft: StartPointDraft) -> Result<StartPoint, RepositoryError>;
    async fn save(&self, start_point: &StartPoint) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<StartPoint>) -> Result<(), RepositoryError>;
    /// Atomically makes `id` the sole default (single UPDATE statement).
    async fn set_default(&self, id: Id<StartPoint>) -> Result<(), RepositoryError>;
}
