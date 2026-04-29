use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    region::Region,
    shared::{
        coordinates::Coordinate,
        field_update::FieldUpdate,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        watering_status::WateringStatus,
    },
    tree::Tree,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "tree_soil_condition", rename_all = "snake_case")]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TreeCluster {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub watering_status: WateringStatus,
    pub last_watered: Option<DateTime<Utc>>,
    pub moisture_level: f64,
    pub region_id: Option<Id<Region>>,
    pub address: String,
    pub description: String,
    pub archived: bool,
    pub coordinates: Option<Coordinate>,
    pub tree_ids: Vec<Id<Tree>>,
    pub soil_condition: Option<SoilCondition>,
    pub name: String,
    pub provider_info: ProviderInfo,
}

#[derive(Debug)]
pub struct TreeClusterCreate {
    pub address: String,
    pub description: String,
    pub name: String,
    pub soil_condition: SoilCondition,
    pub tree_ids: Vec<Id<Tree>>,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct TreeClusterUpdate {
    pub address: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub soil_condition: Option<SoilCondition>,
    pub tree_ids: Option<Vec<Id<Tree>>>,
    pub provider_info: Option<ProviderInfo>,
    pub coordinates: FieldUpdate<Coordinate>,
    pub region_id: FieldUpdate<Id<Region>>,
}

#[derive(Debug, Default)]
pub struct TreeClusterQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub regions: Vec<String>,
    pub ids: Vec<i32>,
    pub provider: Option<String>,
}

#[async_trait::async_trait]
pub trait TreeClusterRepository: Send + Sync {
    async fn all(
        &self,
        query: TreeClusterQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeCluster>, RepositoryError>;
    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<TreeCluster>]) -> Result<Vec<TreeCluster>, RepositoryError>;
    async fn create(&self, entity: TreeClusterCreate) -> Result<TreeCluster, RepositoryError>;
    async fn update(
        &self,
        id: Id<TreeCluster>,
        entity: TreeClusterUpdate,
    ) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn center_point(&self, id: Id<TreeCluster>) -> Result<Coordinate, RepositoryError>;
}
