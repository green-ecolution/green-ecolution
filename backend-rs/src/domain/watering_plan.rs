use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    shared::{distance::Distance, pagination::{Page, Pagination}, provider_info::ProviderInfo},
    vehicle::Vehicle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WateringPlanStatus {
    Planned,
    Active,
    Canceled,
    Finished,
    NotCompleted,
}

#[derive(Debug, Clone)]
pub struct WateringPlan {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub status: WateringPlanStatus,
    pub distance: Option<Distance>,
    pub total_water_required: Option<f64>,
    // pub user_ids: Vec<Id<User>>
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Id<Vehicle>,
    pub trailer_id: Id<Vehicle>,
    pub cancellation_note: Option<String>,
    pub evaluation: Option<WateringPlanStatus>,
    pub gpx_url: url::Url,
    pub refill_count: u32,
    pub duration: Duration,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Clone)]
pub struct WateringPlanEvaluation {
    pub watering_plan_id: Id<WateringPlan>,
    pub cluster_id: Id<TreeCluster>,
    pub consumed_water: f64,
}

#[derive(Debug)]
pub struct WateringPlanCreate {
    pub date: DateTime<Utc>,
    pub description: String,
    pub cluster_ids: Vec<Id<TreeCluster>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    // pub user_ids: Vec<Id<User>>,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct WateringPlanUpdate {
    pub date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub cluster_ids: Option<Vec<Id<TreeCluster>>>,
    pub transporter_id: Option<Id<Vehicle>>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub cancellation_note: Option<String>,
    pub status: Option<WateringPlanStatus>,
    pub evaluation: Option<Vec<WateringPlanEvaluation>>,
    // pub user_ids: Option<Vec<Id<User>>>,
    pub provider_info: Option<ProviderInfo>,
}

#[derive(Debug, Default)]
pub struct WateringPlanQuery {
    pub provider: Option<String>,
}

#[async_trait::async_trait]
pub trait WateringPlanRepository: Send + Sync {
    async fn all(&self, query: WateringPlanQuery, pagination: Pagination) -> Result<Page<WateringPlan>, RepositoryError>;
    async fn count(&self, query: WateringPlanQuery) -> Result<u64, RepositoryError>;
    async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, RepositoryError>;
    async fn create(&self, entity: WateringPlanCreate) -> Result<WateringPlan, RepositoryError>;
    async fn update(
        &self,
        id: Id<WateringPlan>,
        entity: WateringPlanUpdate,
    ) -> Result<WateringPlan, RepositoryError>;
    async fn delete(&self, id: Id<WateringPlan>) -> Result<(), RepositoryError>;
}
