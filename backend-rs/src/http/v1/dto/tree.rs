use serde::{Deserialize, Serialize};

use crate::domain::{sensor::Sensor, tree::Tree};

use super::{WateringStatus, sensor::SensorResponse};
use crate::http::v1::pagination::PaginationRepsonse;

#[derive(Debug, Serialize)]
pub struct TreeResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub species: String,
    pub number: String,
    pub planting_year: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub watering_status: WateringStatus,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_cluster_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor: Option<SensorResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
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
            tree_cluster_id: Some(tree.cluster_id.value()),
            sensor: sensor.map(SensorResponse::from),
            last_watered: tree.last_watered.map(|dt| dt.to_rfc3339()),
            provider: Some(tree.provider_info.provider.clone()),
            additional_information: Some(tree.provider_info.additional_info.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TreeListResponse {
    pub data: Vec<TreeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationRepsonse>,
}

#[derive(Debug, Serialize)]
pub struct TreeWithDistanceResponse {
    pub tree: TreeResponse,
    pub distance_meters: f64,
}

#[derive(Debug, Deserialize)]
pub struct TreeCreateRequest {
    pub species: String,
    pub number: String,
    pub planting_year: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TreeUpdateRequest {
    pub species: String,
    pub number: String,
    pub planting_year: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    #[serde(default)]
    pub tree_cluster_id: Option<i32>,
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}
