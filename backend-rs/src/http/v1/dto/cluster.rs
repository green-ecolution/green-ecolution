use serde::{Deserialize, Serialize};

use crate::domain::{
    Id,
    cluster::{TreeCluster, TreeClusterCreate, TreeClusterUpdate},
    region::Region,
    shared::provider_info::ProviderInfo,
};

use super::{SoilCondition, WateringStatus, region::RegionResponse, tree::TreeResponse};

// -- Responses --

#[derive(Debug, Serialize)]
pub struct TreeClusterResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub address: String,
    pub description: String,
    pub watering_status: WateringStatus,
    pub moisture_level: f64,
    pub soil_condition: SoilCondition,
    pub latitude: f64,
    pub longitude: f64,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<RegionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,
    pub trees: Vec<TreeResponse>,
}

/// Bundles a [TreeCluster] with its resolved relations for conversion
/// into a full [TreeClusterResponse].
pub struct TreeClusterView<'a> {
    pub cluster: &'a TreeCluster,
    pub region: Option<&'a Region>,
    pub trees: Vec<TreeResponse>,
}

impl From<TreeClusterView<'_>> for TreeClusterResponse {
    fn from(view: TreeClusterView<'_>) -> Self {
        let c = view.cluster;
        Self {
            id: c.id.value(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            name: c.name.clone(),
            address: c.address.clone(),
            description: c.description.clone(),
            watering_status: c.watering_status.into(),
            moisture_level: c.moisture_level,
            soil_condition: c
                .soil_condition
                .map(Into::into)
                .unwrap_or(SoilCondition::Unknown),
            latitude: c.coordinates.map(|co| co.latitude()).unwrap_or_default(),
            longitude: c.coordinates.map(|co| co.longitude()).unwrap_or_default(),
            archived: c.archived,
            region: view.region.map(RegionResponse::from),
            provider: Some(c.provider_info.provider.clone()),
            additional_information: Some(c.provider_info.additional_info.clone()),
            last_watered: c.last_watered.map(|dt| dt.to_rfc3339()),
            trees: view.trees,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TreeClusterInListResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub address: String,
    pub description: String,
    pub watering_status: WateringStatus,
    pub moisture_level: f64,
    pub soil_condition: SoilCondition,
    pub latitude: f64,
    pub longitude: f64,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<RegionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,
    pub tree_ids: Vec<i32>,
}

impl From<(&TreeCluster, Option<&Region>)> for TreeClusterInListResponse {
    fn from((c, region): (&TreeCluster, Option<&Region>)) -> Self {
        Self {
            id: c.id.value(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            name: c.name.clone(),
            address: c.address.clone(),
            description: c.description.clone(),
            watering_status: c.watering_status.into(),
            moisture_level: c.moisture_level,
            soil_condition: c
                .soil_condition
                .map(Into::into)
                .unwrap_or(SoilCondition::Unknown),
            latitude: c.coordinates.map(|co| co.latitude()).unwrap_or_default(),
            longitude: c.coordinates.map(|co| co.longitude()).unwrap_or_default(),
            archived: c.archived,
            region: region.map(RegionResponse::from),
            provider: Some(c.provider_info.provider.clone()),
            additional_information: Some(c.provider_info.additional_info.clone()),
            last_watered: c.last_watered.map(|dt| dt.to_rfc3339()),
            tree_ids: c.tree_ids.iter().map(|id| id.value()).collect(),
        }
    }
}

// -- Requests --

#[derive(Debug, Deserialize)]
pub struct TreeClusterCreateRequest {
    pub name: String,
    pub address: String,
    pub description: String,
    pub soil_condition: SoilCondition,
    pub tree_ids: Vec<i32>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TreeClusterUpdateRequest {
    pub name: String,
    pub address: String,
    pub description: String,
    pub soil_condition: SoilCondition,
    pub tree_ids: Vec<i32>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

impl From<TreeClusterCreateRequest> for TreeClusterCreate {
    fn from(req: TreeClusterCreateRequest) -> Self {
        Self {
            name: req.name,
            address: req.address,
            description: req.description,
            soil_condition: req.soil_condition.into(),
            tree_ids: req.tree_ids.into_iter().map(Id::new).collect(),
            provider_info: ProviderInfo {
                provider: req.provider.unwrap_or_default(),
                additional_info: req.additional_information.unwrap_or_default(),
            },
        }
    }
}

impl From<TreeClusterUpdateRequest> for TreeClusterUpdate {
    fn from(req: TreeClusterUpdateRequest) -> Self {
        Self {
            name: Some(req.name),
            address: Some(req.address),
            description: Some(req.description),
            soil_condition: Some(req.soil_condition.into()),
            tree_ids: Some(req.tree_ids.into_iter().map(Id::new).collect()),
            provider_info: Some(ProviderInfo {
                provider: req.provider.unwrap_or_default(),
                additional_info: req.additional_information.unwrap_or_default(),
            }),
            ..Default::default()
        }
    }
}
