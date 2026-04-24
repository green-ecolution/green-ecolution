use chrono::{DateTime, Utc};

use crate::domain::{
    Id, RepositoryError,
    shared::{
        coordinates::Coordinate, pagination::Page, provider_info::ProviderInfo,
        watering_status::WateringStatus,
    },
    tree::Tree,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
}

#[derive(Debug, Clone)]
pub struct TreeCluster {
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    watering_status: WateringStatus,
    last_watered: Option<DateTime<Utc>>,
    moisture_level: f64,
    // region_id: Option<Id<Region>>
    address: String,
    coordinates: Option<Coordinate>,
    tree_ids: Vec<Id<Tree>>,
    soil_condition: Option<SoilCondition>,
    name: String,
    provider_info: ProviderInfo,
}

impl TreeCluster {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        watering_status: WateringStatus,
        last_watered: Option<DateTime<Utc>>,
        moisture_level: f64,
        address: String,
        coordinates: Option<Coordinate>,
        tree_ids: Vec<Id<Tree>>,
        soil_condition: Option<SoilCondition>,
        name: String,
        provider_info: ProviderInfo,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            watering_status,
            last_watered,
            moisture_level,
            address,
            coordinates,
            tree_ids,
            soil_condition,
            name,
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
    pub fn watering_status(&self) -> WateringStatus {
        self.watering_status
    }
    pub fn last_watered(&self) -> Option<DateTime<Utc>> {
        self.last_watered
    }
    pub fn moisture_level(&self) -> f64 {
        self.moisture_level
    }
    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn coordinates(&self) -> Option<&Coordinate> {
        self.coordinates.as_ref()
    }
    pub fn tree_ids(&self) -> &[Id<Tree>] {
        &self.tree_ids
    }
    pub fn soil_condition(&self) -> Option<SoilCondition> {
        self.soil_condition
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn provider_info(&self) -> &ProviderInfo {
        &self.provider_info
    }
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
}

#[derive(Debug, Default)]
pub struct TreeClusterQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub regions: Vec<String>,
    pub ids: Vec<i32>,
    pub provider: Option<String>,
}

#[trait_variant::make(Send)]
pub trait TreeClusterRepository {
    async fn all(&self, query: TreeClusterQuery) -> Result<Page<TreeCluster>, RepositoryError>;
    async fn count(&self, query: TreeClusterQuery) -> Result<u64, RepositoryError>;
    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError>;
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
