use chrono::{DateTime, Datelike, Utc};

use crate::domain::{
    DomainError, Id, RepositoryError,
    cluster::TreeCluster,
    sensor::Sensor,
    shared::{
        coordinates::Coordinate, distance::Distance, pagination::Page, provider_info::ProviderInfo,
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
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    cluster_id: Id<TreeCluster>,
    sensor_id: Id<Sensor>,
    planting_year: PlantingYear,
    species: String,
    tree_number: String,
    coordinate: Coordinate,
    watering_status: WateringStatus,
    description: Option<String>,
    last_watered: Option<DateTime<Utc>>,
    provider_info: ProviderInfo,
}

impl Tree {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        cluster_id: Id<TreeCluster>,
        sensor_id: Id<Sensor>,
        planting_year: PlantingYear,
        species: String,
        tree_number: String,
        coordinate: Coordinate,
        watering_status: WateringStatus,
        description: Option<String>,
        last_watered: Option<DateTime<Utc>>,
        provider_info: ProviderInfo,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            cluster_id,
            sensor_id,
            planting_year,
            species,
            tree_number,
            coordinate,
            watering_status,
            description,
            last_watered,
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
    pub fn cluster_id(&self) -> &Id<TreeCluster> {
        &self.cluster_id
    }
    pub fn sensor_id(&self) -> &Id<Sensor> {
        &self.sensor_id
    }
    pub fn planting_year(&self) -> &PlantingYear {
        &self.planting_year
    }
    pub fn species(&self) -> &str {
        &self.species
    }
    pub fn tree_number(&self) -> &str {
        &self.tree_number
    }
    pub fn coordinate(&self) -> &Coordinate {
        &self.coordinate
    }
    pub fn watering_status(&self) -> WateringStatus {
        self.watering_status
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn last_watered(&self) -> Option<DateTime<Utc>> {
        self.last_watered
    }
    pub fn provider_info(&self) -> &ProviderInfo {
        &self.provider_info
    }
}

pub struct TreeWithDistance {
    tree: Tree,
    distance: Distance,
}

impl TreeWithDistance {
    pub fn new(tree: Tree, distance: Distance) -> Self {
        Self { tree, distance }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }
    pub fn distance(&self) -> Distance {
        self.distance
    }
}

#[derive(Debug)]
pub struct TreeCreate {
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<Id<Sensor>>,
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
    pub sensor_id: Option<Id<Sensor>>,
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
    pub sensor_id: Option<Id<Sensor>>,
    pub provider: Option<String>,
}

#[trait_variant::make(Send)]
pub trait TreeRepository {
    async fn all(&self, query: TreeQuery) -> Result<Page<Tree>, RepositoryError>;
    async fn count(&self, query: TreeQuery) -> Result<u64, RepositoryError>;
    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError>;
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
    async fn unlink_sensor_id(&self, sensor_id: Id<Sensor>) -> Result<(), RepositoryError>;
}
