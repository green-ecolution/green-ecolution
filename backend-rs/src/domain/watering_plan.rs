use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    shared::{distance::Distance, pagination::Page, provider_info::ProviderInfo},
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
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    date: DateTime<Utc>,
    description: Option<String>,
    status: WateringPlanStatus,
    distance: Option<Distance>,
    total_water_required: Option<f64>,
    // user_ids: Vec<Id<User>>
    cluster_ids: Vec<Id<TreeCluster>>,
    transporter_id: Id<Vehicle>,
    trailer_id: Id<Vehicle>,
    cancellation_note: Option<String>,
    evaluation: Option<WateringPlanStatus>,
    gpx_url: url::Url,
    refill_count: u32,
    duration: Duration,
    provider_info: ProviderInfo,
}

impl WateringPlan {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        date: DateTime<Utc>,
        description: Option<String>,
        status: WateringPlanStatus,
        distance: Option<Distance>,
        total_water_required: Option<f64>,
        cluster_ids: Vec<Id<TreeCluster>>,
        transporter_id: Id<Vehicle>,
        trailer_id: Id<Vehicle>,
        cancellation_note: Option<String>,
        evaluation: Option<WateringPlanStatus>,
        gpx_url: url::Url,
        refill_count: u32,
        duration: Duration,
        provider_info: ProviderInfo,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            date,
            description,
            status,
            distance,
            total_water_required,
            cluster_ids,
            transporter_id,
            trailer_id,
            cancellation_note,
            evaluation,
            gpx_url,
            refill_count,
            duration,
            provider_info,
        }
    }

    pub fn id(&self) -> &Id<Self> {
        &self.id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn status(&self) -> WateringPlanStatus {
        self.status
    }
    pub fn distance(&self) -> Option<&Distance> {
        self.distance.as_ref()
    }
    pub fn total_water_required(&self) -> Option<f64> {
        self.total_water_required
    }
    pub fn cluster_ids(&self) -> &[Id<TreeCluster>] {
        &self.cluster_ids
    }
    pub fn transporter_id(&self) -> &Id<Vehicle> {
        &self.transporter_id
    }
    pub fn trailer_id(&self) -> &Id<Vehicle> {
        &self.trailer_id
    }
    pub fn cancellation_note(&self) -> Option<&str> {
        self.cancellation_note.as_deref()
    }
    pub fn evaluation(&self) -> Option<WateringPlanStatus> {
        self.evaluation
    }
    pub fn gpx_url(&self) -> &url::Url {
        &self.gpx_url
    }
    pub fn refill_count(&self) -> u32 {
        self.refill_count
    }
    pub fn duration(&self) -> Duration {
        self.duration
    }
    pub fn provider_info(&self) -> &ProviderInfo {
        &self.provider_info
    }
}

#[derive(Debug, Clone)]
pub struct WateringPlanEvaluation {
    watering_plan_id: Id<WateringPlan>,
    cluster_id: Id<TreeCluster>,
    consumed_water: f64,
}

impl WateringPlanEvaluation {
    pub fn new(
        watering_plan_id: Id<WateringPlan>,
        cluster_id: Id<TreeCluster>,
        consumed_water: f64,
    ) -> Self {
        Self {
            watering_plan_id,
            cluster_id,
            consumed_water,
        }
    }

    pub fn watering_plan_id(&self) -> &Id<WateringPlan> {
        &self.watering_plan_id
    }
    pub fn cluster_id(&self) -> &Id<TreeCluster> {
        &self.cluster_id
    }
    pub fn consumed_water(&self) -> f64 {
        self.consumed_water
    }
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

#[trait_variant::make(Send)]
pub trait WateringPlanRepository {
    async fn all(&self, query: WateringPlanQuery) -> Result<Page<WateringPlan>, RepositoryError>;
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
