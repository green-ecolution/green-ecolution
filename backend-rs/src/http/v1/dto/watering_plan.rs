use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    DomainError, Id,
    shared::provider_info::ProviderInfo,
    watering_plan::{WateringPlan, WateringPlanCreate, WateringPlanUpdate},
};

use super::{WateringPlanStatus, cluster::TreeClusterInListResponse, vehicle::VehicleResponse};

/// Evaluation result for a single tree cluster within a watering plan.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EvaluationValueResponse {
    /// Identifier of the watering plan this evaluation belongs to.
    #[schema(example = 1, minimum = 1)]
    pub watering_plan_id: i32,
    /// Identifier of the tree cluster that was evaluated.
    #[schema(example = 3, minimum = 1)]
    pub tree_cluster_id: i32,
    /// Amount of water consumed for this cluster in liters.
    #[schema(example = 5000.0, minimum = 0.0)]
    pub consumed_water: f64,
}

// -- Detail response --

/// Full detail representation of a watering plan including all related entities.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WateringPlanResponse {
    /// Unique identifier of the watering plan.
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    /// Timestamp when the watering plan was created (RFC 3339).
    #[schema(example = "2024-08-10T09:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the watering plan was last updated (RFC 3339).
    #[schema(example = "2024-08-14T16:30:00+00:00")]
    pub updated_at: String,
    /// Scheduled date and time of the watering run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the watering plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Current status of the watering plan.
    pub status: WateringPlanStatus,
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Total amount of water required for all clusters in liters.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_water_required: f64,
    /// Reason for cancellation, empty if not cancelled.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// URL to the downloadable GPX route file.
    #[schema(example = "/api/v1/watering-plans/route/gpx/route-2024-08-15.gpx")]
    pub gpx_url: String,
    /// Number of water refill stops required during the run.
    #[schema(example = 2, minimum = 0)]
    pub refill_count: u32,
    /// Estimated total duration of the watering run in seconds.
    #[schema(example = 3600.0, minimum = 0.0)]
    pub duration: f64,
    /// The transporter vehicle assigned to this plan.
    pub transporter: VehicleResponse,
    /// The trailer vehicle assigned to this plan, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub trailer: Option<VehicleResponse>,
    /// Tree clusters included in this watering plan.
    pub treeclusters: Vec<TreeClusterInListResponse>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Per-cluster evaluation results after the watering run.
    pub evaluation: Vec<EvaluationValueResponse>,
    /// Name of the external data provider that supplied this record.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
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
            gpx_url: p
                .gpx_url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            refill_count: p.refill_count,
            duration: p.duration.as_secs_f64(),
            transporter: view.transporter,
            trailer: view.trailer,
            treeclusters: view.clusters,
            user_ids: view.user_ids,
            evaluation: view.evaluation,
            provider: p.provider_info.provider.clone(),
            additional_information: p.provider_info.additional_info.clone(),
        }
    }
}

// -- List response --

/// Compact representation of a watering plan used in list endpoints.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WateringPlanInListResponse {
    /// Unique identifier of the watering plan.
    #[schema(example = 1, minimum = 1)]
    pub id: i32,
    /// Timestamp when the watering plan was created (RFC 3339).
    #[schema(example = "2024-08-10T09:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the watering plan was last updated (RFC 3339).
    #[schema(example = "2024-08-14T16:30:00+00:00")]
    pub updated_at: String,
    /// Scheduled date and time of the watering run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the watering plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Current status of the watering plan.
    pub status: WateringPlanStatus,
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Total amount of water required for all clusters in liters.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_water_required: f64,
    /// Reason for cancellation, empty if not cancelled.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// The transporter vehicle assigned to this plan.
    pub transporter: VehicleResponse,
    /// The trailer vehicle assigned to this plan, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub trailer: Option<VehicleResponse>,
    /// Tree clusters included in this watering plan.
    pub treeclusters: Vec<TreeClusterInListResponse>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Name of the external data provider that supplied this record.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
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
            provider: p.provider_info.provider.clone(),
            additional_information: p.provider_info.additional_info.clone(),
        }
    }
}

// -- Requests --

/// Request body for creating a new watering plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WateringPlanCreateRequest {
    /// Scheduled date and time of the watering run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the watering plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = 1, minimum = 1)]
    pub transporter_id: i32,
    /// List of tree cluster identifiers to include in the plan.
    #[schema(example = json!([1, 2, 3]))]
    pub tree_cluster_ids: Vec<i32>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = 2, nullable)]
    pub trailer_id: Option<i32>,
    /// Name of the external data provider that supplied this record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for updating an existing watering plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WateringPlanUpdateRequest {
    /// Scheduled date and time of the watering run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the watering plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Updated status of the watering plan.
    pub status: WateringPlanStatus,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = 1, minimum = 1)]
    pub transporter_id: i32,
    /// List of tree cluster identifiers to include in the plan.
    #[schema(example = json!([1, 2, 3]))]
    pub tree_cluster_ids: Vec<i32>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Reason for cancellation, if applicable.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = 2, nullable)]
    pub trailer_id: Option<i32>,
    /// Per-cluster evaluation results for the watering run.
    #[serde(default)]
    #[schema(nullable)]
    pub evaluation: Option<Vec<EvaluationValueRequest>>,
    /// Name of the external data provider that supplied this record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Evaluation input for a single tree cluster within a watering plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EvaluationValueRequest {
    /// Identifier of the watering plan this evaluation belongs to.
    #[schema(example = 1, minimum = 1)]
    pub watering_plan_id: i32,
    /// Identifier of the tree cluster that was evaluated.
    #[schema(example = 3, minimum = 1)]
    pub tree_cluster_id: i32,
    /// Amount of water consumed for this cluster in liters.
    #[schema(example = 5000.0, minimum = 0.0)]
    pub consumed_water: f64,
}

/// Request body for calculating a watering route.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RouteRequest {
    /// List of tree cluster identifiers to include in the route.
    #[schema(example = json!([1, 2, 3]))]
    pub cluster_ids: Vec<i32>,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = 1, minimum = 1)]
    pub transporter_id: i32,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = 2, nullable)]
    pub trailer_id: Option<i32>,
}

fn parse_date(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| DomainError::InvalidInput(format!("invalid date: {e}")))
}

impl TryFrom<WateringPlanCreateRequest> for WateringPlanCreate {
    type Error = DomainError;

    fn try_from(req: WateringPlanCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            date: parse_date(&req.date)?,
            description: req.description,
            cluster_ids: req.tree_cluster_ids.into_iter().map(Id::new).collect(),
            transporter_id: Some(Id::new(req.transporter_id)),
            trailer_id: req.trailer_id.map(Id::new),
            provider_info: ProviderInfo {
                provider: req.provider,
                additional_info: req.additional_information,
            },
        })
    }
}

impl TryFrom<WateringPlanUpdateRequest> for WateringPlanUpdate {
    type Error = DomainError;

    fn try_from(req: WateringPlanUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            date: Some(parse_date(&req.date)?),
            description: Some(req.description),
            cluster_ids: Some(req.tree_cluster_ids.into_iter().map(Id::new).collect()),
            transporter_id: Some(Id::new(req.transporter_id)),
            trailer_id: req.trailer_id.map(Id::new),
            cancellation_note: Some(req.cancellation_note),
            status: Some(req.status.into()),
            evaluation: None,
            provider_info: Some(ProviderInfo {
                provider: req.provider,
                additional_info: req.additional_information,
            }),
        })
    }
}
