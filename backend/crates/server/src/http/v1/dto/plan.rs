use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::{Malformed, ServiceError};
use domain::{
    Id,
    module::ModuleKey,
    operation_plan::{OperationPlanDraft, OperationPlanStatus, OperationPlanView, StationVisit},
    organization::Organization,
    planning::{StopOutcome, StopRef},
    shared::{
        coordinates::Coordinate,
        provenance::{Provenance, ProviderId},
    },
};

use super::vehicle::VehicleResponse;

/// Lifecycle status of a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "planned")]
pub enum PlanStatus {
    /// Plan has been created but not yet started.
    Planned,
    /// Plan is currently being executed.
    Active,
    /// Plan was canceled before completion.
    Canceled,
    /// Plan was completed successfully.
    Finished,
    /// Plan was started but could not be completed.
    #[serde(alias = "not competed")]
    NotCompleted,
}

impl From<PlanStatus> for OperationPlanStatus {
    fn from(value: PlanStatus) -> Self {
        match value {
            PlanStatus::Planned => Self::Planned,
            PlanStatus::Active => Self::Active,
            PlanStatus::Canceled => Self::Canceled,
            PlanStatus::Finished => Self::Finished,
            PlanStatus::NotCompleted => Self::NotCompleted,
        }
    }
}

impl From<OperationPlanStatus> for PlanStatus {
    fn from(value: OperationPlanStatus) -> Self {
        match value {
            OperationPlanStatus::Planned => Self::Planned,
            OperationPlanStatus::Active => Self::Active,
            OperationPlanStatus::Canceled => Self::Canceled,
            OperationPlanStatus::Finished => Self::Finished,
            OperationPlanStatus::NotCompleted => Self::NotCompleted,
        }
    }
}

/// Query parameters for the paginated plan list endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PlanListParams {
    /// Required: plans always belong to exactly one module.
    #[param(example = "irrigation")]
    pub module: String,
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
    pub status: Vec<PlanStatus>,
}

/// Result recorded for one stop when a plan is finished.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StopOutcomeDto {
    /// The stop, as the plan's module identifies it.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub stop: Uuid,
    /// Whether the stop was served.
    #[schema(example = true)]
    pub completed: bool,
    /// Amount the stop took, in the unit of the module's capacity kind.
    #[schema(example = 120.0, nullable)]
    pub amount: Option<f64>,
}

impl From<StopOutcome> for StopOutcomeDto {
    fn from(o: StopOutcome) -> Self {
        Self {
            stop: o.stop.value(),
            completed: o.completed,
            amount: o.amount,
        }
    }
}

impl From<StopOutcomeDto> for StopOutcome {
    fn from(o: StopOutcomeDto) -> Self {
        Self {
            stop: StopRef::new(o.stop),
            completed: o.completed,
            amount: o.amount,
        }
    }
}

// -- Detail response --

/// Full detail representation of a plan including its vehicles and outcomes.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanResponse {
    /// Unique identifier of the plan.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: Uuid,
    /// Module the plan belongs to.
    #[schema(example = "irrigation")]
    pub module: String,
    /// Timestamp when the plan was created (RFC 3339).
    #[schema(example = "2024-08-10T09:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the plan was last updated (RFC 3339).
    #[schema(example = "2024-08-14T16:30:00+00:00")]
    pub updated_at: String,
    /// Scheduled date and time of the run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Current status of the plan.
    pub status: PlanStatus,
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Summed demand of all stops, in the unit of the module's capacity kind.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_demand: f64,
    /// Reason for cancellation, empty if not cancelled.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// URL to the downloadable GPX route file.
    #[schema(example = "/v1/plans/0190a8e9-7c4f-7000-8000-000000000000/route/gpx")]
    pub gpx_url: String,
    /// Number of station stops required during the run.
    #[schema(example = 2, minimum = 0)]
    pub refill_count: u32,
    /// Estimated total duration of the run in seconds.
    #[schema(example = 3600.0, minimum = 0.0)]
    pub duration: f64,
    /// The transporter vehicle assigned to this plan.
    pub transporter: VehicleResponse,
    /// The trailer vehicle assigned to this plan, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub trailer: Option<VehicleResponse>,
    /// Whether the transporter's own capacity counts towards the route capacity.
    /// `false` only occurs with a trailer.
    #[schema(example = true)]
    pub transporter_capacity_used: bool,
    /// Stops of the plan, as the plan's module identifies them.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub stops: Vec<Uuid>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Per-stop results; not completed and without amount until the plan is finished.
    pub outcomes: Vec<StopOutcomeDto>,
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
    /// Identifier of the organization this plan belongs to.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: String,
}

pub struct PlanDetailView {
    pub view: OperationPlanView,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
    pub outcomes: Vec<StopOutcomeDto>,
}

impl From<PlanDetailView> for PlanResponse {
    fn from(d: PlanDetailView) -> Self {
        let v = &d.view;
        Self {
            id: v.id,
            module: v.module.to_string(),
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
            date: v.date.to_rfc3339(),
            description: v.description.clone().unwrap_or_default(),
            status: v.status.into(),
            distance: v.distance.unwrap_or_default(),
            total_demand: v.total_demand.unwrap_or_default(),
            cancellation_note: v.cancellation_note.clone().unwrap_or_default(),
            // Path is relative to the frontend's API basePath; GPX is
            // rendered on the fly from the stored route geometry.
            gpx_url: if v.distance.is_some() {
                format!("/v1/plans/{}/route/gpx", v.id)
            } else {
                String::new()
            },
            refill_count: v.refill_count.max(0) as u32,
            duration: v.duration.as_secs_f64(),
            transporter: d.transporter,
            trailer: d.trailer,
            transporter_capacity_used: v.transporter_capacity_used,
            stops: v.stops.clone(),
            user_ids: v.user_ids.iter().map(Uuid::to_string).collect(),
            outcomes: d.outcomes,
            start_point_name: v.start_point_name.clone(),
            provider: v.provider.clone(),
            additional_information: v.additional_info.clone(),
            organization_id: v.organization_id.to_string(),
        }
    }
}

// -- List response --

/// Compact representation of a plan used in list endpoints.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanInListResponse {
    /// Unique identifier of the plan.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: Uuid,
    /// Module the plan belongs to.
    #[schema(example = "irrigation")]
    pub module: String,
    /// Timestamp when the plan was created (RFC 3339).
    #[schema(example = "2024-08-10T09:00:00+00:00")]
    pub created_at: String,
    /// Timestamp when the plan was last updated (RFC 3339).
    #[schema(example = "2024-08-14T16:30:00+00:00")]
    pub updated_at: String,
    /// Scheduled date and time of the run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Current status of the plan.
    pub status: PlanStatus,
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Summed demand of all stops, in the unit of the module's capacity kind.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_demand: f64,
    /// Reason for cancellation, empty if not cancelled.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// The transporter vehicle assigned to this plan.
    pub transporter: VehicleResponse,
    /// The trailer vehicle assigned to this plan, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub trailer: Option<VehicleResponse>,
    /// Whether the transporter's own capacity counts towards the route capacity.
    /// `false` only occurs with a trailer.
    #[schema(example = true)]
    pub transporter_capacity_used: bool,
    /// Stops of the plan, as the plan's module identifies them.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub stops: Vec<Uuid>,
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

pub struct PlanInListDetailView {
    pub view: OperationPlanView,
    pub transporter: VehicleResponse,
    pub trailer: Option<VehicleResponse>,
}

impl From<PlanInListDetailView> for PlanInListResponse {
    fn from(d: PlanInListDetailView) -> Self {
        let v = &d.view;
        Self {
            id: v.id,
            module: v.module.to_string(),
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
            date: v.date.to_rfc3339(),
            description: v.description.clone().unwrap_or_default(),
            status: v.status.into(),
            distance: v.distance.unwrap_or_default(),
            total_demand: v.total_demand.unwrap_or_default(),
            cancellation_note: v.cancellation_note.clone().unwrap_or_default(),
            transporter: d.transporter,
            trailer: d.trailer,
            transporter_capacity_used: v.transporter_capacity_used,
            stops: v.stops.clone(),
            user_ids: v.user_ids.iter().map(Uuid::to_string).collect(),
            provider: v.provider.clone(),
            additional_information: v.additional_info.clone(),
        }
    }
}

// -- Requests --

/// Request body for creating a new plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PlanCreateRequest {
    /// Module the plan belongs to.
    #[schema(example = "irrigation")]
    pub module: String,
    /// Scheduled date and time of the run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: Uuid,
    /// Stops to include in the plan, as the module identifies them.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub stops: Vec<Uuid>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<Uuid>,
    /// Whether the transporter's own capacity counts towards the route capacity.
    /// May only be `false` together with a `trailer_id`.
    #[serde(default = "default_true")]
    #[schema(example = true, default = true)]
    pub transporter_capacity_used: bool,
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
    /// Organization this plan belongs to. Defaults to the acting user's own
    /// organization when omitted.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub organization_id: Option<Uuid>,
}

/// Request body for updating an existing plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PlanUpdateRequest {
    /// Scheduled date and time of the run (RFC 3339).
    #[schema(example = "2024-08-15T06:00:00+00:00")]
    pub date: String,
    /// Human-readable description of the plan.
    #[schema(example = "Bewässerungsplan Stadtpark Nord")]
    pub description: String,
    /// Updated status of the plan.
    pub status: PlanStatus,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: Uuid,
    /// Stops to include in the plan, as the module identifies them.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub stops: Vec<Uuid>,
    /// List of user identifiers assigned to execute this plan.
    #[schema(example = json!(["user-uuid-1"]))]
    pub user_ids: Vec<String>,
    /// Reason for cancellation, if applicable.
    #[schema(example = "Regen vorhergesagt")]
    pub cancellation_note: String,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<Uuid>,
    /// Whether the transporter's own capacity counts towards the route capacity.
    /// May only be `false` together with a `trailer_id`.
    #[serde(default = "default_true")]
    #[schema(example = true, default = true)]
    pub transporter_capacity_used: bool,
    /// Name of the configured start point to depart from and return to. Unknown names fall back to the default depot.
    #[serde(default)]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
    /// One result per stop; required when the status moves to `finished`.
    #[serde(default)]
    #[schema(nullable)]
    pub outcomes: Option<Vec<StopOutcomeDto>>,
    /// Name of the external data provider that supplied this record.
    #[serde(default)]
    #[schema(example = "green_ecolution", nullable)]
    pub provider: Option<String>,
    /// Arbitrary additional metadata from the provider.
    #[serde(default)]
    #[schema(value_type = Object, nullable)]
    pub additional_information: Option<serde_json::Value>,
}

/// Request body for calculating a route without creating a plan.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PlanRouteRequest {
    /// Module whose stops are routed.
    #[schema(example = "irrigation")]
    pub module: String,
    /// Stops to include in the route, as the module identifies them.
    #[schema(example = json!(["0190a8e9-7c4f-7000-8000-000000000000"]))]
    pub stops: Vec<Uuid>,
    /// Identifier of the transporter vehicle to use.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub transporter_id: Uuid,
    /// Identifier of the trailer vehicle to use, if any.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub trailer_id: Option<Uuid>,
    /// Whether the transporter's own capacity counts towards the route capacity.
    /// May only be `false` together with a `trailer_id`.
    #[serde(default = "default_true")]
    #[schema(example = true, default = true)]
    pub transporter_capacity_used: bool,
    /// Name of the configured start point to depart from and return to. Unknown names fall back to the default depot.
    #[serde(default)]
    #[schema(example = "Betriebshof Schleswiger Straße", nullable)]
    pub start_point_name: Option<String>,
    /// Organization the plan will belong to; start points and stations are
    /// taken from its organization chain. Defaults to the acting user's own
    /// organization when omitted, like a plan created without one.
    #[serde(default)]
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    pub organization_id: Option<Uuid>,
}

fn default_true() -> bool {
    true
}

/// GeoJSON LineString geometry of an optimized route.
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

/// Station an optimized route visits to restore its capacity.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StationVisitResponse {
    /// Name of the station at route computation time.
    #[schema(example = "Klärwerk Kielseng")]
    pub name: String,
    /// Latitude of the station (WGS84).
    #[schema(example = 54.8052)]
    pub lat: f64,
    /// Longitude of the station (WGS84).
    #[schema(example = 9.4471)]
    pub lon: f64,
}

impl From<&StationVisit> for StationVisitResponse {
    fn from(p: &StationVisit) -> Self {
        Self {
            name: p.name.as_str().to_owned(),
            lat: p.coordinate.latitude(),
            lon: p.coordinate.longitude(),
        }
    }
}

/// Optimized route (persisted for a plan, or a preview).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RouteResponse {
    /// Total route distance in meters.
    #[schema(example = 12500.0, minimum = 0.0)]
    pub distance: f64,
    /// Estimated total driving duration in seconds.
    #[schema(example = 3600.0, minimum = 0.0)]
    pub duration: f64,
    /// Number of station stops on the route.
    #[schema(example = 2, minimum = 0)]
    pub refill_count: u32,
    /// Stations the route actually visits, in visit order.
    pub station_visits: Vec<StationVisitResponse>,
    /// Summed demand of all routed stops, in the unit of the module's capacity kind.
    #[schema(example = 24000.0, minimum = 0.0)]
    pub total_demand: f64,
    /// Route geometry as a GeoJSON LineString.
    pub geometry: RouteGeometry,
}

pub(crate) fn parse_date(s: &str) -> Result<DateTime<Utc>, ServiceError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ServiceError::Malformed {
            kind: Malformed::Date,
            detail: e.to_string(),
        })
}

pub(crate) fn parse_user_ids(ids: &[String]) -> Result<Vec<Uuid>, ServiceError> {
    ids.iter()
        .map(|s| {
            s.parse::<Uuid>().map_err(|e| ServiceError::Malformed {
                kind: Malformed::Uuid,
                detail: format!("{s}: {e}"),
            })
        })
        .collect()
}

pub(crate) fn stop_refs(ids: &[Uuid]) -> Vec<StopRef> {
    ids.iter().copied().map(StopRef::new).collect()
}

impl PlanCreateRequest {
    pub fn into_draft(
        self,
        module: ModuleKey,
        organization_id: Id<Organization>,
    ) -> Result<OperationPlanDraft, ServiceError> {
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
        Ok(OperationPlanDraft {
            module,
            date,
            description,
            start_point_name: self.start_point_name,
            stops: stop_refs(&self.stops),
            transporter_id: Some(Id::new(self.transporter_id)),
            trailer_id: self.trailer_id.map(Id::new),
            transporter_capacity_used: self.transporter_capacity_used,
            provenance,
            user_ids: parse_user_ids(&self.user_ids)?,
            organization_id,
        })
    }
}
