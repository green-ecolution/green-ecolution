use chrono::{DateTime, Datelike, Utc};

use crate::domain::{
    DomainError, Id, RepositoryError,
    cluster::TreeCluster,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        watering_status::WateringStatus,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantingYear(u32);

impl PlantingYear {
    pub fn new(year: u32) -> Result<Self, DomainError> {
        let current_year = Utc::now().year() as u32;
        if year > current_year {
            return Err(DomainError::InvalidPlantingYear(year));
        }
        Ok(Self(year))
    }

    pub fn year(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub id: Id<Self>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<String>,
    pub planting_year: PlantingYear,
    pub species: String,
    pub tree_number: String,
    pub coordinate: Coordinate,
    pub watering_status: WateringStatus,
    pub description: Option<String>,
    pub last_watered: Option<DateTime<Utc>>,
    pub provider_info: ProviderInfo,
}

pub struct TreeWithDistance {
    pub tree: Tree,
    pub distance: Distance,
}

#[derive(Debug)]
pub struct TreeCreate {
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<String>,
    pub planting_year: PlantingYear,
    pub species: String,
    pub tree_number: String,
    pub coordinate: Coordinate,
    pub description: String,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct TreeUpdate {
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<String>,
    pub planting_year: Option<PlantingYear>,
    pub species: Option<String>,
    pub tree_number: Option<String>,
    pub coordinate: Option<Coordinate>,
    pub description: Option<String>,
    pub provider_info: Option<ProviderInfo>,
}

#[derive(Debug, Default)]
pub struct TreeQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub has_cluster: Option<bool>,
    pub planting_years: Vec<u32>,
    pub ids: Vec<Id<Self>>,
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<String>,
    pub provider: Option<String>,
}

#[async_trait::async_trait]
pub trait TreeRepository: Send + Sync {
    async fn all(
        &self,
        query: TreeQuery,
        pagination: Pagination,
    ) -> Result<Page<Tree>, RepositoryError>;
    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError>;
    async fn create(&self, entity: TreeCreate) -> Result<Tree, RepositoryError>;
    async fn update(&self, id: Id<Tree>, entity: TreeUpdate) -> Result<Tree, RepositoryError>;
    async fn archive(&self, id: Id<Tree>) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError>;

    async fn nearest_trees(
        &self,
        coord: Coordinate,
        radius_meters: f64,
        limit: u32,
    ) -> Result<Vec<TreeWithDistance>, RepositoryError>;

    async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, RepositoryError>;
    async fn unlink_cluster_id(&self, cluster_id: Id<TreeCluster>) -> Result<(), RepositoryError>;
    async fn unlink_sensor_id(&self, sensor_id: &str) -> Result<(), RepositoryError>;
}
