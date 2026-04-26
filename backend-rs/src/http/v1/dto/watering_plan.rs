use serde::{Deserialize, Serialize};

use crate::domain::watering_plan::WateringPlan;

use super::{WateringPlanStatus, cluster::TreeClusterInListResponse, vehicle::VehicleResponse};
use crate::http::v1::pagination::PaginationRepsonse;

#[derive(Debug, Serialize)]
pub struct EvaluationValueResponse {
    pub watering_plan_id: i32,
    pub tree_cluster_id: i32,
    pub consumed_water: f64,
}

// -- Detail response --

#[derive(Debug, Serialize)]
pub struct WateringPlanResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub date: String,
    pub description: String,
    pub status: WateringPlanStatus,
    pub distance: f64,
    pub total_water_required: f64,
    pub cancellation_note: String,
    pub gpx_url: String,
    pub refill_count: u32,
    pub duration: f64,
    pub transporter: VehicleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<VehicleResponse>,
    pub treeclusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
    pub evaluation: Vec<EvaluationValueResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

pub struct WateringPlanView {
    pub plan: WateringPlan,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
    pub clusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
    pub evaluation: Vec<EvaluationValueResponse>,
}

impl From<WateringPlanView> for WateringPlanResponse {
    fn from(view: WateringPlanView) -> Self {
        let p = &view.plan;
        Self {
            id: p.id.value(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
            date: p.date.to_rfc3339(),
            description: p.description.clone().unwrap_or_default(),
            status: p.status.into(),
            distance: p.distance.map(|d| d.meters()).unwrap_or_default(),
            total_water_required: p.total_water_required.unwrap_or_default(),
            cancellation_note: p.cancellation_note.clone().unwrap_or_default(),
            gpx_url: p.gpx_url.to_string(),
            refill_count: p.refill_count,
            duration: p.duration.as_secs_f64(),
            transporter: view.transporter,
            trailer: view.trailer,
            treeclusters: view.clusters,
            user_ids: view.user_ids,
            evaluation: view.evaluation,
            provider: Some(p.provider_info.provider.clone()),
            additional_information: Some(p.provider_info.additional_info.clone()),
        }
    }
}

// -- List response --

#[derive(Debug, Serialize)]
pub struct WateringPlanInListResponse {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub date: String,
    pub description: String,
    pub status: WateringPlanStatus,
    pub distance: f64,
    pub total_water_required: f64,
    pub cancellation_note: String,
    pub transporter: VehicleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<VehicleResponse>,
    pub treeclusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

pub struct WateringPlanInListView {
    pub plan: WateringPlan,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
    pub clusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
}

impl From<WateringPlanInListView> for WateringPlanInListResponse {
    fn from(view: WateringPlanInListView) -> Self {
        let p = &view.plan;
        Self {
            id: p.id.value(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
            date: p.date.to_rfc3339(),
            description: p.description.clone().unwrap_or_default(),
            status: p.status.into(),
            distance: p.distance.map(|d| d.meters()).unwrap_or_default(),
            total_water_required: p.total_water_required.unwrap_or_default(),
            cancellation_note: p.cancellation_note.clone().unwrap_or_default(),
            transporter: view.transporter,
            trailer: view.trailer,
            treeclusters: view.clusters,
            user_ids: view.user_ids,
            provider: Some(p.provider_info.provider.clone()),
            additional_information: Some(p.provider_info.additional_info.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WateringPlanListResponse {
    pub data: Vec<WateringPlanInListResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationRepsonse>,
}

// -- Requests --

#[derive(Debug, Deserialize)]
pub struct WateringPlanCreateRequest {
    pub date: String,
    pub description: String,
    pub transporter_id: i32,
    pub tree_cluster_ids: Vec<i32>,
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub trailer_id: Option<i32>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct WateringPlanUpdateRequest {
    pub date: String,
    pub description: String,
    pub status: WateringPlanStatus,
    pub transporter_id: i32,
    pub tree_cluster_ids: Vec<i32>,
    pub user_ids: Vec<String>,
    pub cancellation_note: String,
    #[serde(default)]
    pub trailer_id: Option<i32>,
    #[serde(default)]
    pub evaluation: Option<Vec<EvaluationValueRequest>>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluationValueRequest {
    pub watering_plan_id: i32,
    pub tree_cluster_id: i32,
    pub consumed_water: f64,
}

#[derive(Debug, Deserialize)]
pub struct RouteRequest {
    pub cluster_ids: Vec<i32>,
    pub transporter_id: i32,
    #[serde(default)]
    pub trailer_id: Option<i32>,
}
