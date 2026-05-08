use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::service::ServiceError;
use domain::{
    Id,
    shared::provenance::{Provenance, ProviderId},
    watering_plan::{WateringPlanDraft, WateringPlanEvaluation, WateringPlanView},
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

pub struct WateringPlanDetailView {
    pub view: WateringPlanView,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
    pub clusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
    pub evaluation: Vec<EvaluationValueResponse>,
}

impl From<WateringPlanDetailView> for WateringPlanResponse {
    fn from(d: WateringPlanDetailView) -> Self {
        let v = &d.view;
        Self {
            id: v.id,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
            date: v.date.to_rfc3339(),
            description: v.description.clone().unwrap_or_default(),
            status: v.status.into(),
            distance: v.distance.unwrap_or_default(),
            total_water_required: v.total_water_required.unwrap_or_default(),
            cancellation_note: v.cancellation_note.clone().unwrap_or_default(),
            gpx_url: v
                .gpx_url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            refill_count: v.refill_count.max(0) as u32,
            duration: v.duration.as_secs_f64(),
            transporter: d.transporter,
            trailer: d.trailer,
            treeclusters: d.clusters,
            user_ids: d.user_ids,
            evaluation: d.evaluation,
            provider: v.provider.clone(),
            additional_information: v.additional_info.clone(),
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

pub struct WateringPlanInListDetailView {
    pub view: WateringPlanView,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
    pub clusters: Vec<TreeClusterInListResponse>,
    pub user_ids: Vec<String>,
}

impl From<WateringPlanInListDetailView> for WateringPlanInListResponse {
    fn from(d: WateringPlanInListDetailView) -> Self {
        let v = &d.view;
        Self {
            id: v.id,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
            date: v.date.to_rfc3339(),
            description: v.description.clone().unwrap_or_default(),
            status: v.status.into(),
            distance: v.distance.unwrap_or_default(),
            total_water_required: v.total_water_required.unwrap_or_default(),
            cancellation_note: v.cancellation_note.clone().unwrap_or_default(),
            transporter: d.transporter,
            trailer: d.trailer,
            treeclusters: d.clusters,
            user_ids: d.user_ids,
            provider: v.provider.clone(),
            additional_information: v.additional_info.clone(),
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

impl EvaluationValueRequest {
    pub fn into_domain(
        self,
        plan_id: Id<domain::watering_plan::WateringPlan>,
    ) -> WateringPlanEvaluation {
        WateringPlanEvaluation {
            watering_plan_id: plan_id,
            cluster_id: Id::new(self.tree_cluster_id),
            consumed_water: self.consumed_water,
        }
    }
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

fn parse_date(s: &str) -> Result<DateTime<Utc>, ServiceError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ServiceError::InvalidInput(format!("invalid date: {e}")))
}

impl TryFrom<WateringPlanCreateRequest> for WateringPlanDraft {
    type Error = ServiceError;

    fn try_from(req: WateringPlanCreateRequest) -> Result<Self, Self::Error> {
        let date = parse_date(&req.date)?;
        let provenance = Provenance::new(
            req.provider.map(ProviderId::new).transpose()?,
            req.additional_information,
        );
        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description)
        };
        Ok(Self {
            date,
            description,
            cluster_ids: req.tree_cluster_ids.into_iter().map(Id::new).collect(),
            transporter_id: Some(Id::new(req.transporter_id)),
            trailer_id: req.trailer_id.map(Id::new),
            provenance,
        })
    }
}
