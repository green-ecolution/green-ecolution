use serde::{Deserialize, Serialize};

use domain::{
    Id,
    organization::Organization,
    sensor::{SensorId, SensorView},
    shared::{
        coordinates::Coordinate,
        geo::BoundingBox,
        provenance::{Provenance, ProviderId},
    },
    tree::{PlantingYear, Species, TreeDraft, TreeMarker, TreeNumber, TreeView},
};

use crate::service::ServiceError;

use super::{WateringStatus, sensor::SensorResponse};

/// An individual tree managed by the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_cluster_id: Option<uuid::Uuid>,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: String,
    /// Organizations this tree is shared with, in addition to its owning
    /// organization. If the tree belongs to a cluster, this also includes
    /// organizations the cluster itself is shared with.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub shared_with: Vec<String>,
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
            organization_id: tree.organization_id.to_string(),
            shared_with: tree.shared_with.iter().map(ToString::to_string).collect(),
        }
    }
}

/// Request body for sharing a resource with a descendant organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ShareRequest {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: uuid::Uuid,
}

/// A tree with its distance from a reference point.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeWithDistanceResponse {
    pub tree: TreeResponse,
    #[schema(example = 142.5, minimum = 0.0)]
    pub distance_meters: f64,
}

/// Query parameters for the paginated tree list endpoint.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct TreeListParams {
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "crate::http::v1::pagination::default_page")]
    pub page: u64,
    #[param(default = 25, minimum = 1, maximum = 100, example = 25)]
    #[serde(default = "crate::http::v1::pagination::default_per_page")]
    pub per_page: u64,
    #[param(example = "Eiche")]
    pub q: Option<String>,
    /// Repeatable: `?watering_status=good&watering_status=bad`.
    #[serde(default)]
    pub watering_status: Vec<WateringStatus>,
    #[param(nullable)]
    #[serde(default)]
    pub has_cluster: Option<bool>,
    /// Repeatable: `?planting_year=2018&planting_year=2020`.
    #[serde(default)]
    pub planting_year: Vec<i32>,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<uuid::Uuid>,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}

impl TreeCreateRequest {
    pub fn into_draft(self, organization_id: Id<Organization>) -> Result<TreeDraft, ServiceError> {
        Ok(TreeDraft {
            cluster_id: self.tree_cluster_id.map(Id::new),
            sensor_id: self.sensor_id.map(SensorId::new).transpose()?,
            planting_year: PlantingYear::new(self.planting_year as u32)?,
            species: Species::new(self.species)?,
            tree_number: TreeNumber::new(self.number)?,
            coordinate: Coordinate::new(self.latitude, self.longitude)?,
            description: if self.description.is_empty() {
                None
            } else {
                Some(self.description)
            },
            provenance: Provenance::new(
                self.provider.map(ProviderId::new).transpose()?,
                self.additional_information,
            ),
            organization_id,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<uuid::Uuid>,
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

/// Lightweight tree marker for the map.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeMarkerResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    pub watering_status: WateringStatus,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    pub has_sensor: bool,
}

impl From<&TreeMarker> for TreeMarkerResponse {
    fn from(m: &TreeMarker) -> Self {
        Self {
            id: m.id,
            latitude: m.latitude,
            longitude: m.longitude,
            watering_status: m.watering_status.into(),
            number: m.tree_number.clone(),
            has_sensor: m.has_sensor,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeMarkerListResponse {
    pub data: Vec<TreeMarkerResponse>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TreeMarkerQueryParams {
    /// Format: `swLat,swLng,neLat,neLng`. Required.
    #[param(example = "54.78,9.40,54.81,9.46")]
    pub bbox: String,
    #[param(nullable)]
    #[serde(default)]
    pub has_cluster: Option<bool>,
    /// Repeatable: `?planting_year=2018&planting_year=2020`.
    #[serde(default)]
    pub planting_year: Vec<i32>,
    /// Repeatable: `?watering_status=good&watering_status=bad`.
    #[serde(default)]
    pub watering_status: Vec<WateringStatus>,
}

impl TreeMarkerQueryParams {
    pub fn parse_bbox(&self) -> Result<BoundingBox, String> {
        let parts: Vec<&str> = self.bbox.split(',').collect();
        if parts.len() != 4 {
            return Err(format!(
                "bbox must be 'swLat,swLng,neLat,neLng' (got {} parts)",
                parts.len()
            ));
        }
        let sw_lat: f64 = parts[0]
            .trim()
            .parse()
            .map_err(|e| format!("sw_lat: {e}"))?;
        let sw_lng: f64 = parts[1]
            .trim()
            .parse()
            .map_err(|e| format!("sw_lng: {e}"))?;
        let ne_lat: f64 = parts[2]
            .trim()
            .parse()
            .map_err(|e| format!("ne_lat: {e}"))?;
        let ne_lng: f64 = parts[3]
            .trim()
            .parse()
            .map_err(|e| format!("ne_lng: {e}"))?;
        BoundingBox::try_new(sw_lat, sw_lng, ne_lat, ne_lng).map_err(|e| e.to_string())
    }
}
