use serde::{Deserialize, Serialize};

use domain::{
    Id,
    sensor::{SensorId, SensorView},
    shared::{
        coordinates::Coordinate,
        error::ValidationError,
        provenance::{Provenance, ProviderId},
    },
    tree::{PlantingYear, Species, TreeDraft, TreeNumber, TreeView},
};

use super::{WateringStatus, sensor::SensorResponse};

/// An individual tree managed by the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "2024-01-15T10:30:00+00:00")]
    pub created_at: String,
    #[schema(example = "2024-06-20T14:00:00+00:00")]
    pub updated_at: String,
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    pub watering_status: WateringStatus,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = 5, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_cluster_id: Option<i32>,
    #[schema(nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor: Option<SensorResponse>,
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

impl From<(&TreeView, Option<&SensorView>)> for TreeResponse {
    fn from((tree, sensor): (&TreeView, Option<&SensorView>)) -> Self {
        Self {
            id: tree.id,
            created_at: tree.created_at.to_rfc3339(),
            updated_at: tree.updated_at.to_rfc3339(),
            species: tree.species.clone(),
            number: tree.tree_number.clone(),
            planting_year: tree.planting_year as i32,
            latitude: tree.latitude,
            longitude: tree.longitude,
            watering_status: tree.watering_status.into(),
            description: tree.description.clone().unwrap_or_default(),
            tree_cluster_id: tree.cluster_id,
            sensor: sensor.map(SensorResponse::from),
            last_watered: tree.last_watered.map(|dt| dt.to_rfc3339()),
            provider: tree.provider.clone(),
            additional_information: tree.additional_info.clone(),
        }
    }
}

/// A tree with its distance from a reference point.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeWithDistanceResponse {
    pub tree: TreeResponse,
    #[schema(example = 142.5, minimum = 0.0)]
    pub distance_meters: f64,
}

/// Query parameters for finding the nearest trees to a given point.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct NearestTreeParams {
    #[param(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub lat: f64,
    #[param(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub lng: f64,
    #[param(example = 10, minimum = 1, maximum = 100)]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Non-paginated list of nearest trees with distance information.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NearestTreeListResponse {
    pub data: Vec<TreeWithDistanceResponse>,
}

/// Request body for creating a new tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeCreateRequest {
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = 5, nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

impl TryFrom<TreeCreateRequest> for TreeDraft {
    type Error = ValidationError;

    fn try_from(req: TreeCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            cluster_id: req.tree_cluster_id.map(Id::new),
            sensor_id: req.sensor_id.map(SensorId::new).transpose()?,
            planting_year: PlantingYear::new(req.planting_year as u32)?,
            species: Species::new(req.species)?,
            tree_number: TreeNumber::new(req.number)?,
            coordinate: Coordinate::new(req.latitude, req.longitude)?,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            provenance: Provenance::new(
                req.provider.map(ProviderId::new).transpose()?,
                req.additional_information,
            ),
        })
    }
}

/// Request body for updating an existing tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeUpdateRequest {
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = 5, nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}
