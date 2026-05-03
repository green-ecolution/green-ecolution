use async_trait::async_trait;

use crate::domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    shared::pagination::{Page, Pagination},
    watering_plan::{
        WateringPlan, WateringPlanDraft, WateringPlanEvaluation, WateringPlanSearchQuery,
        WateringPlanView,
    },
};

#[async_trait]
pub trait WateringPlanReader: Send + Sync {
    async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, RepositoryError>;
    async fn view_by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlanView, RepositoryError>;
    async fn view_search(
        &self,
        query: WateringPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlanView>, RepositoryError>;
    async fn evaluations(
        &self,
        plan_id: Id<WateringPlan>,
    ) -> Result<Vec<WateringPlanEvaluation>, RepositoryError>;
}

#[async_trait]
pub trait WateringPlanWriter: Send + Sync {
    async fn save_new(&self, draft: WateringPlanDraft) -> Result<WateringPlan, RepositoryError>;
    async fn save(&self, plan: &WateringPlan) -> Result<(), RepositoryError>;
    async fn save_evaluations(
        &self,
        plan_id: Id<WateringPlan>,
        evaluations: &[WateringPlanEvaluation],
    ) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<WateringPlan>) -> Result<(), RepositoryError>;
    async fn propagate_last_watered(
        &self,
        cluster_ids: &[Id<TreeCluster>],
        ts: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError>;
}
