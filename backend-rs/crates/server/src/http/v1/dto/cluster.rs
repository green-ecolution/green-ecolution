use serde::{Deserialize, Serialize};

use crate::domain::{
    Id,
    cluster::{ClusterAddress, ClusterName, TreeCluster, TreeClusterDraft, TreeClusterView},
    region::Region,
    shared::{
        error::ValidationError,
        provenance::{Provenance, ProviderId},
    },
};

use super::{SoilCondition, WateringStatus, region::RegionResponse, tree::TreeResponse};

/// Full representation of a tree cluster including its resolved tree relations.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeClusterResponse {
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    #[schema(example = "2024-06-15T12:00:00+00:00")]
    pub created_at: String,
    #[schema(example = "2024-07-10T08:30:00+00:00")]
    pub updated_at: String,
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    pub watering_status: WateringStatus,
    #[schema(example = 0.65, minimum = 0.0, maximum = 1.0)]
    pub moisture_level: f64,
    pub soil_condition: SoilCondition,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = false)]
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub region: Option<RegionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    pub last_watered: Option<String>,
    pub trees: Vec<TreeResponse>,
}

impl TreeClusterResponse {
    pub fn from_parts(
        view: &TreeClusterView,
        region: Option<&Region>,
        trees: Vec<TreeResponse>,
    ) -> Self {
        Self {
            id: view.id,
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
            name: view.name.clone(),
            address: view.address.clone(),
            description: view.description.clone(),
            watering_status: view.watering_status.into(),
            moisture_level: view.moisture_level,
            soil_condition: view
                .soil_condition
                .map(Into::into)
                .unwrap_or(SoilCondition::Unknown),
            latitude: view.latitude.unwrap_or_default(),
            longitude: view.longitude.unwrap_or_default(),
            archived: view.archived,
            region: region.map(RegionResponse::from),
            provider: view.provider.clone(),
            additional_information: view.additional_info.clone(),
            last_watered: view.last_watered.map(|dt| dt.to_rfc3339()),
            trees,
        }
    }
}

/// Compact representation of a tree cluster used in list endpoints (tree IDs instead of full objects).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeClusterInListResponse {
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    #[schema(example = "2024-06-15T12:00:00+00:00")]
    pub created_at: String,
    #[schema(example = "2024-07-10T08:30:00+00:00")]
    pub updated_at: String,
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    pub watering_status: WateringStatus,
    #[schema(example = 0.65, minimum = 0.0, maximum = 1.0)]
    pub moisture_level: f64,
    pub soil_condition: SoilCondition,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = false)]
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub region: Option<RegionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    pub last_watered: Option<String>,
    #[schema(example = json!([1, 2, 3]))]
    pub tree_ids: Vec<i32>,
}

impl From<(&TreeClusterView, Option<&Region>)> for TreeClusterInListResponse {
    fn from((view, region): (&TreeClusterView, Option<&Region>)) -> Self {
        Self {
            id: view.id,
            created_at: view.created_at.to_rfc3339(),
            updated_at: view.updated_at.to_rfc3339(),
            name: view.name.clone(),
            address: view.address.clone(),
            description: view.description.clone(),
            watering_status: view.watering_status.into(),
            moisture_level: view.moisture_level,
            soil_condition: view
                .soil_condition
                .map(Into::into)
                .unwrap_or(SoilCondition::Unknown),
            latitude: view.latitude.unwrap_or_default(),
            longitude: view.longitude.unwrap_or_default(),
            archived: view.archived,
            region: region.map(RegionResponse::from),
            provider: view.provider.clone(),
            additional_information: view.additional_info.clone(),
            last_watered: view.last_watered.map(|dt| dt.to_rfc3339()),
            tree_ids: view.tree_ids.clone(),
        }
    }
}

impl From<(&TreeCluster, Option<&Region>)> for TreeClusterInListResponse {
    fn from((c, region): (&TreeCluster, Option<&Region>)) -> Self {
        Self {
            id: c.id.value(),
            created_at: String::new(),
            updated_at: String::new(),
            name: c.name.as_str().to_owned(),
            address: c.address.as_str().to_owned(),
            description: c.description.clone(),
            watering_status: c.watering_status().into(),
            moisture_level: c.moisture_level,
            soil_condition: c
                .soil_condition
                .map(Into::into)
                .unwrap_or(SoilCondition::Unknown),
            latitude: c.coordinates().map(|co| co.latitude()).unwrap_or_default(),
            longitude: c.coordinates().map(|co| co.longitude()).unwrap_or_default(),
            archived: c.archived(),
            region: region.map(RegionResponse::from),
            provider: c.provenance().provider().map(|p| p.as_str().to_owned()),
            additional_information: c.provenance().additional_info().cloned(),
            last_watered: c.last_watered.map(|dt| dt.to_rfc3339()),
            tree_ids: c.tree_ids.iter().map(|id| id.value()).collect(),
        }
    }
}

// -- Requests --

/// Request body for creating a new tree cluster.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeClusterCreateRequest {
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    pub soil_condition: SoilCondition,
    #[schema(example = json!([1, 2, 3]))]
    pub tree_ids: Vec<i32>,
    #[serde(default)]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for updating an existing tree cluster.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeClusterUpdateRequest {
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    pub soil_condition: SoilCondition,
    #[schema(example = json!([1, 2, 3]))]
    pub tree_ids: Vec<i32>,
    #[serde(default)]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

impl TryFrom<TreeClusterCreateRequest> for TreeClusterDraft {
    type Error = ValidationError;

    fn try_from(req: TreeClusterCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            name: ClusterName::new(req.name)?,
            address: ClusterAddress::new(req.address)?,
            description: req.description,
            moisture_level: 0.0,
            soil_condition: Some(req.soil_condition.into()),
            tree_ids: req.tree_ids.into_iter().map(Id::new).collect(),
            provenance: Provenance::new(
                req.provider.map(ProviderId::new).transpose()?,
                req.additional_information,
            ),
        })
    }
}
