use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    operation_plan::{ItineraryEntry, OperationPlanStatus},
    shared::coordinates::Coordinate,
};

/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OperationPlanSnapshot {
    pub id: Uuid,
    pub module: String,
    pub date: DateTime<Utc>,
    pub description: Option<String>,
    pub start_point_name: Option<String>,
    pub status: OperationPlanStatus,
    pub distance: Option<f64>,
    pub total_demand: Option<f64>,
    pub stops: Vec<Uuid>,
    pub user_ids: Vec<Uuid>,
    pub transporter_id: Option<Uuid>,
    pub trailer_id: Option<Uuid>,
    pub transporter_capacity_used: bool,
    pub cancellation_note: Option<String>,
    pub refill_count: i32,
    pub duration: Duration,
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
    pub route_geometry: Option<Vec<Coordinate>>,
    pub itinerary: Vec<ItineraryEntry>,
    pub organization_id: Uuid,
}
