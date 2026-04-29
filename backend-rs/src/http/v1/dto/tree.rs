use serde::{Deserialize, Serialize};

use crate::domain::{
    DomainError, Id,
    sensor::Sensor,
    shared::{coordinates::Coordinate, provider_info::ProviderInfo},
    tree::{PlantingYear, Tree, TreeCreate, TreeUpdate},
};

use super::{WateringStatus, sensor::SensorResponse};

/// An individual tree managed by the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeResponse {
    /// Unique tree identifier.
    #[schema(example = 1)]
    pub id: i32,

    /// Timestamp when the tree was registered (RFC 3339).
    #[schema(example = "2024-01-15T10:30:00+00:00")]
    pub created_at: String,

    /// Timestamp of the last update (RFC 3339).
    #[schema(example = "2024-06-20T14:00:00+00:00")]
    pub updated_at: String,

    /// Tree species name.
    #[schema(example = "Quercus robur")]
    pub species: String,

    /// Internal tree identification number.
    #[schema(example = "T-2024-0042")]
    pub number: String,

    /// Year the tree was planted.
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,

    /// Geographic latitude (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,

    /// Geographic longitude (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,

    /// Current watering status based on sensor data.
    pub watering_status: WateringStatus,

    /// Additional notes or description.
    #[schema(example = "Standort nahe Spielplatz, guter Boden")]
    pub description: String,

    /// ID of the cluster this tree belongs to, if assigned.
    #[schema(example = 5, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_cluster_id: Option<i32>,

    /// Associated sensor, if linked.
    #[schema(nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor: Option<SensorResponse>,

    /// Timestamp of the last watering event (RFC 3339).
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,

    /// Name of the data provider that created this tree.
    #[schema(example = "green-ecolution", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Provider-specific metadata as arbitrary JSON.
    #[schema(value_type = Object, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

impl From<(&Tree, Option<&Sensor>)> for TreeResponse {
    fn from((tree, sensor): (&Tree, Option<&Sensor>)) -> Self {
        Self {
            id: tree.id.value(),
            created_at: tree.created_at.to_rfc3339(),
            updated_at: tree.updated_at.to_rfc3339(),
            species: tree.species.clone(),
            number: tree.tree_number.clone(),
            planting_year: tree.planting_year.year() as i32,
            latitude: tree.coordinate.latitude(),
            longitude: tree.coordinate.longitude(),
            watering_status: tree.watering_status.into(),
            description: tree.description.clone().unwrap_or_default(),
            tree_cluster_id: tree.cluster_id.as_ref().map(|id| id.value()),
            sensor: sensor.map(SensorResponse::from),
            last_watered: tree.last_watered.map(|dt| dt.to_rfc3339()),
            provider: tree.provider_info.provider.clone(),
            additional_information: tree.provider_info.additional_info.clone(),
        }
    }
}

/// A tree with its distance from a reference point.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeWithDistanceResponse {
    /// The tree data.
    pub tree: TreeResponse,

    /// Distance from the reference point in meters.
    #[schema(example = 142.5, minimum = 0.0)]
    pub distance_meters: f64,
}

/// Query parameters for finding the nearest trees to a given point.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct NearestTreeParams {
    /// Latitude of the reference point (WGS 84).
    #[param(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub lat: f64,

    /// Longitude of the reference point (WGS 84).
    #[param(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub lng: f64,

    /// Maximum number of results to return.
    #[param(example = 10, minimum = 1, maximum = 100)]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Non-paginated list of nearest trees with distance information.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NearestTreeListResponse {
    /// List of trees sorted by distance (ascending).
    pub data: Vec<TreeWithDistanceResponse>,
}

/// Request body for creating a new tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeCreateRequest {
    /// Tree species name.
    #[schema(example = "Quercus robur")]
    pub species: String,

    /// Internal tree identification number.
    #[schema(example = "T-2024-0042")]
    pub number: String,

    /// Year the tree was planted.
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,

    /// Geographic latitude (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,

    /// Geographic longitude (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,

    /// Additional notes or description.
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,

    /// Cluster ID to assign this tree to.
    #[schema(example = 5, nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,

    /// Sensor ID to link to this tree.
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,

    /// Name of the data provider creating this tree.
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,

    /// Provider-specific metadata as arbitrary JSON.
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for updating an existing tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeUpdateRequest {
    /// Tree species name.
    #[schema(example = "Quercus robur")]
    pub species: String,

    /// Internal tree identification number.
    #[schema(example = "T-2024-0042")]
    pub number: String,

    /// Year the tree was planted.
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,

    /// Geographic latitude (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,

    /// Geographic longitude (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,

    /// Additional notes or description.
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,

    /// Cluster ID to assign this tree to.
    #[schema(example = 5, nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,

    /// Sensor ID to link to this tree.
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,

    /// Name of the data provider.
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,

    /// Provider-specific metadata as arbitrary JSON.
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

impl TryFrom<TreeCreateRequest> for TreeCreate {
    type Error = DomainError;

    fn try_from(req: TreeCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            cluster_id: req.tree_cluster_id.map(Id::new),
            sensor_id: req.sensor_id,
            planting_year: PlantingYear::new(req.planting_year as u32)?,
            species: req.species,
            tree_number: req.number,
            coordinate: Coordinate::new(req.latitude, req.longitude)?,
            description: req.description,
            provider_info: ProviderInfo {
                provider: req.provider,
                additional_info: req.additional_information,
            },
        })
    }
}

impl TryFrom<TreeUpdateRequest> for TreeUpdate {
    type Error = DomainError;

    fn try_from(req: TreeUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            cluster_id: req.tree_cluster_id.map(Id::new),
            sensor_id: req.sensor_id,
            planting_year: Some(PlantingYear::new(req.planting_year as u32)?),
            species: Some(req.species),
            tree_number: Some(req.number),
            coordinate: Some(Coordinate::new(req.latitude, req.longitude)?),
            description: Some(req.description),
            provider_info: Some(ProviderInfo {
                provider: req.provider,
                additional_info: req.additional_information,
            }),
        })
    }
}
