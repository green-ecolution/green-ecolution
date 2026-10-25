use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    module::ModuleKey,
    operation_plan::{
        OperationPlan, OperationPlanDraft, OperationPlanSearchQuery, OperationPlanView,
    },
    planning::{StopOutcome, StopRef},
    shared::pagination::{Page, Pagination},
};

/// Read-side access to operation plans, including aggregate hydration and the
/// HTTP-friendly [`OperationPlanView`] read model.
#[async_trait]
pub trait OperationPlanReader: Send + Sync {
    async fn by_id(&self, id: Id<OperationPlan>) -> Result<OperationPlan, RepositoryError>;
    async fn view_by_id(&self, id: Id<OperationPlan>)
    -> Result<OperationPlanView, RepositoryError>;
    async fn view_search(
        &self,
        query: OperationPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<OperationPlanView>, RepositoryError>;
    async fn outcomes(&self, id: Id<OperationPlan>) -> Result<Vec<StopOutcome>, RepositoryError>;
}

/// Write-side access to operation plans.
#[async_trait]
pub trait OperationPlanWriter: Send + Sync {
    async fn save_new(&self, draft: OperationPlanDraft) -> Result<OperationPlan, RepositoryError>;
    async fn save(&self, plan: &OperationPlan) -> Result<(), RepositoryError>;
    /// Persists the finished plan and its outcomes atomically.
    async fn save_finished(
        &self,
        plan: &OperationPlan,
        outcomes: &[StopOutcome],
    ) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<OperationPlan>) -> Result<(), RepositoryError>;
    /// Removes `stop` from every plan of `module`, finished ones included.
    /// Returns how many plans lost the stop.
    async fn remove_stop(&self, module: &ModuleKey, stop: StopRef) -> Result<u64, RepositoryError>;
}
