use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use domain::{
    Id,
    cluster::{
        ClusterAddress, ClusterBoundaryView, ClusterMarker, ClusterName, ClusterStatistics,
        TreeCluster, TreeClusterDraft, TreeClusterView,
    },
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub tree_ids: Vec<uuid::Uuid>,
    #[serde(rename = "sensorCount")]
    #[schema(example = 3, minimum = 0)]
    pub sensor_count: i64,
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
            sensor_count: view.sensor_count,
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
            sensor_count: 0,
        }
    }
}

/// Query parameters for the paginated cluster list endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ClusterListParams {
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "crate::http::v1::pagination::default_page")]
    pub page: u64,
    #[param(default = 25, minimum = 1, maximum = 100, example = 25)]
    #[serde(default = "crate::http::v1::pagination::default_per_page")]
    pub per_page: u64,
    /// Repeatable: `?watering_status=good&watering_status=bad`.
    #[serde(default)]
    pub watering_status: Vec<WateringStatus>,
    /// Repeatable: `?region=<uuid>&region=<uuid>`.
    #[serde(default)]
    pub region: Vec<uuid::Uuid>,
    /// Repeatable: `?soil_condition=Ss&soil_condition=Sl2`.
    #[serde(default)]
    pub soil_condition: Vec<SoilCondition>,
    /// Free-text search across cluster name and address.
    #[param(example = "Hafen")]
    pub query: Option<String>,
    /// Sort field. Allowed: `name|moisture|trees`.
    #[param(example = "name")]
    pub sort: Option<String>,
    /// Sort direction. Allowed: `asc|desc`.
    #[param(example = "asc")]
    pub order: Option<String>,
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
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub tree_ids: Vec<uuid::Uuid>,
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
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub tree_ids: Vec<uuid::Uuid>,
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

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterMarkerResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = "Stadtpark")]
    pub name: String,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    pub watering_status: super::WateringStatus,
    #[schema(example = 12, minimum = 0)]
    pub tree_count: u32,
}

impl From<&ClusterMarker> for ClusterMarkerResponse {
    fn from(m: &ClusterMarker) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            latitude: m.latitude,
            longitude: m.longitude,
            watering_status: m.watering_status.into(),
            tree_count: m.tree_count,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterMarkerListResponse {
    pub data: Vec<ClusterMarkerResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterBoundaryResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = "Stadtpark")]
    pub name: String,
    pub watering_status: super::WateringStatus,
    /// GeoJSON polygon (convex hull of the cluster's trees, buffered in meters).
    #[schema(value_type = Object)]
    pub boundary: serde_json::Value,
}

impl From<&ClusterBoundaryView> for ClusterBoundaryResponse {
    fn from(b: &ClusterBoundaryView) -> Self {
        Self {
            id: b.cluster_id,
            name: b.name.clone(),
            watering_status: b.watering_status.into(),
            boundary: b.boundary.clone(),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterBoundaryListResponse {
    pub data: Vec<ClusterBoundaryResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterStatisticsResponse {
    pub total: i64,
    pub trees: i64,
    pub bad: i64,
    pub moderate: i64,
    pub good: i64,
    #[serde(rename = "justWatered")]
    pub just_watered: i64,
    pub unknown: i64,
}

impl From<ClusterStatistics> for ClusterStatisticsResponse {
    fn from(v: ClusterStatistics) -> Self {
        Self {
            total: v.total,
            trees: v.trees,
            bad: v.bad,
            moderate: v.moderate,
            good: v.good,
            just_watered: v.just_watered,
            unknown: v.unknown,
        }
    }
}

/// Query parameters for the cluster soil-moisture series.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SoilMoistureParams {
    /// Window start (RFC 3339). Defaults to 7 days before `to`.
    pub from: Option<DateTime<Utc>>,
    /// Window end (RFC 3339). Defaults to now.
    pub to: Option<DateTime<Utc>>,
    /// Aggregation bucket. Allowed: `hour|day`. Defaults to `day`.
    #[param(example = "day")]
    pub bucket: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoisturePointResponse {
    /// Bucket start (RFC 3339).
    pub timestamp: DateTime<Utc>,
    #[schema(example = 23.4)]
    pub mean: f64,
    #[schema(example = 21.0)]
    pub min: f64,
    #[schema(example = 25.1)]
    pub max: f64,
    #[schema(example = 12)]
    pub sample_count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoistureDepthSeriesResponse {
    #[schema(example = 40)]
    pub depth_cm: i32,
    pub points: Vec<SoilMoisturePointResponse>,
}

/// Volumetric thresholds (Vol.-%) below which a depth counts as stressed.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoistureThresholdResponse {
    #[schema(example = 40)]
    pub depth_cm: i32,
    #[schema(example = 12.4)]
    pub moderate: f64,
    #[schema(example = 10.9)]
    pub critical: f64,
}

/// One bucket of the tree-condition series, as % plant-available water
/// (REW × 100). The depth with the lowest mean REW wins per bucket.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoistureConditionPointResponse {
    /// Bucket start (RFC 3339).
    pub timestamp: DateTime<Utc>,
    #[schema(example = 46.5)]
    pub mean: f64,
    #[schema(example = 40.0)]
    pub min: f64,
    #[schema(example = 55.0)]
    pub max: f64,
    /// Depth whose mean REW was lowest in this bucket.
    #[schema(example = 80)]
    pub worst_depth_cm: i32,
}

/// Universal REW thresholds in % — identical for every soil type and depth.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoistureConditionThresholdsResponse {
    #[schema(example = 40.0)]
    pub moderate: f64,
    #[schema(example = 30.0)]
    pub critical: f64,
}

/// A finished watering-plan run that included this cluster.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ClusterWateringEventResponse {
    pub watering_plan_id: uuid::Uuid,
    pub date: NaiveDate,
    #[schema(example = 1800.0)]
    pub consumed_water_liters: f64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SoilMoistureSeriesResponse {
    /// Effective bucket: `hour` or `day`.
    #[schema(example = "day")]
    pub bucket: String,
    pub series: Vec<SoilMoistureDepthSeriesResponse>,
    /// Empty when the cluster's soil condition is unknown.
    pub thresholds: Vec<SoilMoistureThresholdResponse>,
    /// Worst-case condition series; empty when the soil condition is unknown.
    pub condition: Vec<SoilMoistureConditionPointResponse>,
    pub condition_thresholds: SoilMoistureConditionThresholdsResponse,
    /// All finished runs, newest first (not limited to the query window).
    pub watering_events: Vec<ClusterWateringEventResponse>,
}

impl From<domain::cluster::SoilMoistureOverview> for SoilMoistureSeriesResponse {
    fn from(v: domain::cluster::SoilMoistureOverview) -> Self {
        Self {
            bucket: match v.bucket {
                domain::cluster::SoilMoistureBucket::Hour => "hour".into(),
                domain::cluster::SoilMoistureBucket::Day => "day".into(),
            },
            series: v
                .series
                .into_iter()
                .map(|s| SoilMoistureDepthSeriesResponse {
                    depth_cm: s.depth_cm,
                    points: s
                        .points
                        .into_iter()
                        .map(|p| SoilMoisturePointResponse {
                            timestamp: p.bucket_start,
                            mean: p.mean,
                            min: p.min,
                            max: p.max,
                            sample_count: p.sample_count,
                        })
                        .collect(),
                })
                .collect(),
            thresholds: v
                .thresholds
                .into_iter()
                .map(|t| SoilMoistureThresholdResponse {
                    depth_cm: t.depth_cm,
                    moderate: t.moderate,
                    critical: t.critical,
                })
                .collect(),
            condition: v
                .condition
                .into_iter()
                .map(|c| SoilMoistureConditionPointResponse {
                    timestamp: c.bucket_start,
                    mean: c.rew_mean * 100.0,
                    min: c.rew_min * 100.0,
                    max: c.rew_max * 100.0,
                    worst_depth_cm: c.worst_depth_cm,
                })
                .collect(),
            condition_thresholds: SoilMoistureConditionThresholdsResponse {
                moderate: domain::tree::REW_MIN * 100.0,
                critical: domain::tree::REW_CRIT * 100.0,
            },
            watering_events: v
                .watering_events
                .into_iter()
                .map(|e| ClusterWateringEventResponse {
                    watering_plan_id: e.watering_plan_id,
                    date: e.date,
                    consumed_water_liters: e.consumed_water_liters,
                })
                .collect(),
        }
    }
}
