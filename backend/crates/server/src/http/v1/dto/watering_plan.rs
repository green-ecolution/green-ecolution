use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::service::ServiceError;
use domain::{
    Id,
    organization::Organization,
    shared::{
        coordinates::Coordinate,
        provenance::{Provenance, ProviderId},
    },
    watering_plan::{RefillPoint, WateringPlanDraft, WateringPlanEvaluation, WateringPlanView},
};

use super::{WateringPlanStatus, cluster::TreeClusterInListResponse, vehicle::VehicleResponse};

/// Query parameters for the paginated watering plan list endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct WateringPlanListParams {
    /// Page number to retrieve (1-based).
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "crate::http::v1::pagination::default_page")]
    pub page: u64,
    /// Number of items per page.
    #[param(default = 25, minimum = 1, maximum = 100, example = 25)]
    #[serde(default = "crate::http::v1::pagination::default_per_page")]
    pub per_page: u64,
    /// Repeatable: `?status=planned&status=active`.
    #[serde(default)]
    pub status: Vec<WateringPlanStatus>,
}

/// Evaluation result for a single tree cluster within a watering plan.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EvaluationValueResponse {
    /// Identifier of the watering plan this evaluation belongs to.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub watering_plan_id: uuid::Uuid,
    /// Identifier of the tree cluster that was evaluated.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub tree_cluster_id: uuid::Uuid,
    /// Amount of water consumed for this cluster in liters.
    #[schema(example = 5000.0, minimum = 0.0)]
    pub consumed_water: f64,
}

// -- Detail response --

/// Full detail representation of a watering plan including all related entities.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WateringPlanResponse {
    /// Unique identifier of the watering plan.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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
    #[schema(example = "/v1/watering-plans/0190a8e9-7c4f-7000-8000-000000000000/route/gpx")]
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
    /// Named start/return point for the route.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
    /// Name of the external data provider that supplied this record.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    /// Identifier of the organization this watering plan belongs to.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: String,
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
            // Path is relative to the frontend's API basePath; GPX is
            // rendered on the fly from the stored route geometry.
            gpx_url: if v.distance.is_some() {
                format!("/v1/watering-plans/{}/route/gpx", v.id)
            } else {
                String::new()
            },
            refill_count: v.refill_count.max(0) as u32,
            duration: v.duration.as_secs_f64(),
            transporter: d.transporter,
            trailer: d.trailer,
            treeclusters: d.clusters,
            user_ids: d.user_ids,
            evaluation: d.evaluation,
            start_point_name: v.start_point_name.clone(),
            provider: v.provider.clone(),
            additional_information: v.additional_info.clone(),
            organization_id: v.organization_id.to_string(),
        }
    }
}

// -- List response --

/// Compact representation of a watering plan used in list endpoints.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WateringPlanInListResponse {
    /// Unique identifier of the watering plan.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: uuid::Uuid,
    /// List of tree cluster identifiers to include in the plan.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub tree_cluster_ids: Vec<uuid::Uuid>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<uuid::Uuid>,
    /// Name of the configured start point to depart from and return to. Unknown names fall back to the default depot.
    #[serde(default)]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
    /// Name of the external data provider that supplied this record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
    /// Organization this watering plan belongs to. Defaults to the acting
    /// user's own organization when omitted.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub organization_id: Option<uuid::Uuid>,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: uuid::Uuid,
    /// List of tree cluster identifiers to include in the plan.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub tree_cluster_ids: Vec<uuid::Uuid>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Reason for cancellation, if applicable.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<uuid::Uuid>,
    /// Name of the configured start point to depart from and return to. Unknown names fall back to the default depot.
    #[serde(default)]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
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
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub watering_plan_id: uuid::Uuid,
    /// Identifier of the tree cluster that was evaluated.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub tree_cluster_id: uuid::Uuid,
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
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub cluster_ids: Vec<uuid::Uuid>,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: uuid::Uuid,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<uuid::Uuid>,
    /// Name of the configured start point to depart from and return to. Unknown names fall back to the default depot.
    #[serde(default)]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
}

/// GeoJSON LineString geometry of an optimized watering route.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RouteGeometry {
    /// GeoJSON geometry type, always "LineString".
    #[schema(example = "LineString")]
    pub r#type: String,
    /// Route coordinates as GeoJSON positions (longitude, latitude).
    #[schema(example = json!([[9.4347, 54.7687], [9.4358, 54.7922]]))]
    pub coordinates: Vec<[f64; 2]>,
}

impl RouteGeometry {
    pub fn from_coordinates(coords: &[Coordinate]) -> Self {
        Self {
            r#type: "LineString".to_string(),
            coordinates: coords
                .iter()
                .map(|c| [c.longitude(), c.latitude()])
                .collect(),
        }
    }
}

/// Water refill station an optimized route visits.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefillPointResponse {
    /// Name of the refill station at route computation time.
    #[schema(example = "Klärwerk Kielseng")]
    pub name: String,
    /// Latitude of the station (WGS84).
    #[schema(example = 54.8052)]
    pub lat: f64,
    /// Longitude of the station (WGS84).
    #[schema(example = 9.4471)]
    pub lon: f64,
}

impl From<&RefillPoint> for RefillPointResponse {
    fn from(p: &RefillPoint) -> Self {
        Self {
            name: p.name.as_str().to_owned(),
            lat: p.coordinate.latitude(),
            lon: p.coordinate.longitude(),
        }
    }
}

/// Optimized watering route (persisted for a plan, or a preview).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RouteResponse {
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Estimated total driving duration in seconds.
    #[schema(example = 3600.0, minimum = 0.0)]
    pub duration: f64,
    /// Number of water refill stops on the route.
    #[schema(example = 2, minimum = 0)]
    pub refill_count: u32,
    /// Water refill stations the route actually visits, in visit order.
    pub refill_points: Vec<RefillPointResponse>,
    /// Total water demand of all routed clusters in liters.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_water_required: f64,
    /// Route geometry as a GeoJSON LineString.
    pub geometry: RouteGeometry,
}

fn parse_date(s: &str) -> Result<DateTime<Utc>, ServiceError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ServiceError::InvalidInput(format!("invalid date: {e}")))
}

pub(crate) fn parse_user_ids(ids: &[String]) -> Result<Vec<uuid::Uuid>, ServiceError> {
    ids.iter()
        .map(|s| {
            s.parse::<uuid::Uuid>()
                .map_err(|e| ServiceError::InvalidInput(format!("invalid user id '{s}': {e}")))
        })
        .collect()
}

impl WateringPlanCreateRequest {
    pub fn into_draft(
        self,
        organization_id: Id<Organization>,
    ) -> Result<WateringPlanDraft, ServiceError> {
        let date = parse_date(&self.date)?;
        let provenance = Provenance::new(
            self.provider.map(ProviderId::new).transpose()?,
            self.additional_information,
        );
        let description = if self.description.is_empty() {
            None
        } else {
            Some(self.description)
        };
        Ok(WateringPlanDraft {
            date,
            description,
            start_point_name: self.start_point_name,
            cluster_ids: self.tree_cluster_ids.into_iter().map(Id::new).collect(),
            transporter_id: Some(Id::new(self.transporter_id)),
            trailer_id: self.trailer_id.map(Id::new),
            provenance,
            user_ids: parse_user_ids(&self.user_ids)?,
            organization_id,
        })
    }
}
