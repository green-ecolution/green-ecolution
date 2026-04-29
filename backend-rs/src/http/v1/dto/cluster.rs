use serde::{Deserialize, Serialize};

use crate::domain::{
    Id,
    cluster::{TreeCluster, TreeClusterCreate, TreeClusterUpdate},
    region::Region,
    shared::provider_info::ProviderInfo,
};

use super::{SoilCondition, WateringStatus, region::RegionResponse, tree::TreeResponse};

// -- Responses --

/// Full representation of a tree cluster including its resolved tree relations.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeClusterResponse {
    /// Unique identifier of the tree cluster.
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    /// Timestamp when the cluster was created (RFC 3339).
    #[schema(example = "2024-06-15T12:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the cluster was last updated (RFC 3339).
    #[schema(example = "2024-07-10T08:30:00+00:00")]
    pub updated_at: String,
    /// Human-readable name of the cluster.
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    /// Street address or location description.
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    /// Longer description of the cluster and its trees.
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    /// Current watering status derived from sensor data.
    pub watering_status: WateringStatus,
    /// Average soil-moisture level across cluster sensors (0.0 = dry, 1.0 = saturated).
    #[schema(example = 0.65, minimum = 0.0, maximum = 1.0)]
    pub moisture_level: f64,
    /// Dominant soil condition of the cluster area.
    pub soil_condition: SoilCondition,
    /// Latitude of the cluster centroid (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    /// Longitude of the cluster centroid (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    /// Whether the cluster has been archived and is no longer actively managed.
    #[schema(example = false)]
    pub archived: bool,
    /// Region the cluster belongs to, if assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub region: Option<RegionResponse>,
    /// Identifier of the data provider that created this cluster.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    /// Provider-specific metadata as a free-form JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    /// Timestamp of the last watering event (RFC 3339), if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    pub last_watered: Option<String>,
    /// Full tree objects belonging to this cluster.
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
            provider: c.provider_info.provider.clone(),
            additional_information: c.provider_info.additional_info.clone(),
            last_watered: c.last_watered.map(|dt| dt.to_rfc3339()),
            trees: view.trees,
        }
    }
}

/// Compact representation of a tree cluster used in list endpoints (tree IDs instead of full objects).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeClusterInListResponse {
    /// Unique identifier of the tree cluster.
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    /// Timestamp when the cluster was created (RFC 3339).
    #[schema(example = "2024-06-15T12:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the cluster was last updated (RFC 3339).
    #[schema(example = "2024-07-10T08:30:00+00:00")]
    pub updated_at: String,
    /// Human-readable name of the cluster.
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    /// Street address or location description.
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    /// Longer description of the cluster and its trees.
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    /// Current watering status derived from sensor data.
    pub watering_status: WateringStatus,
    /// Average soil-moisture level across cluster sensors (0.0 = dry, 1.0 = saturated).
    #[schema(example = 0.65, minimum = 0.0, maximum = 1.0)]
    pub moisture_level: f64,
    /// Dominant soil condition of the cluster area.
    pub soil_condition: SoilCondition,
    /// Latitude of the cluster centroid (WGS 84).
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    /// Longitude of the cluster centroid (WGS 84).
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    /// Whether the cluster has been archived and is no longer actively managed.
    #[schema(example = false)]
    pub archived: bool,
    /// Region the cluster belongs to, if assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub region: Option<RegionResponse>,
    /// Identifier of the data provider that created this cluster.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    /// Provider-specific metadata as a free-form JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    /// Timestamp of the last watering event (RFC 3339), if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    pub last_watered: Option<String>,
    /// IDs of trees belonging to this cluster.
    #[schema(example = json!([1, 2, 3]))]
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
            provider: c.provider_info.provider.clone(),
            additional_information: c.provider_info.additional_info.clone(),
            last_watered: c.last_watered.map(|dt| dt.to_rfc3339()),
            tree_ids: c.tree_ids.iter().map(|id| id.value()).collect(),
        }
    }
}

// -- Requests --

/// Request body for creating a new tree cluster.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeClusterCreateRequest {
    /// Human-readable name of the cluster.
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    /// Street address or location description.
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    /// Longer description of the cluster and its trees.
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    /// Soil condition of the cluster area.
    pub soil_condition: SoilCondition,
    /// IDs of existing trees to assign to this cluster.
    #[schema(example = json!([1, 2, 3]))]
    pub tree_ids: Vec<i32>,
    /// Identifier of the data provider creating this cluster.
    #[serde(default)]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    /// Provider-specific metadata as a free-form JSON object.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for updating an existing tree cluster.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeClusterUpdateRequest {
    /// Human-readable name of the cluster.
    #[schema(example = "Cluster Stadtpark Nord")]
    pub name: String,
    /// Street address or location description.
    #[schema(example = "Stadtpark 1, 24937 Flensburg")]
    pub address: String,
    /// Longer description of the cluster and its trees.
    #[schema(example = "Baumgruppe im nördlichen Parkbereich")]
    pub description: String,
    /// Soil condition of the cluster area.
    pub soil_condition: SoilCondition,
    /// IDs of existing trees to assign to this cluster.
    #[schema(example = json!([1, 2, 3]))]
    pub tree_ids: Vec<i32>,
    /// Identifier of the data provider updating this cluster.
    #[serde(default)]
    #[schema(example = "green-ecolution", nullable)]
    pub provider: Option<String>,
    /// Provider-specific metadata as a free-form JSON object.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
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
                provider: req.provider,
                additional_info: req.additional_information,
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
                provider: req.provider,
                additional_info: req.additional_information,
            }),
            ..Default::default()
        }
    }
}
