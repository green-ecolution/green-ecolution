use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{module::ModuleKey, operation_plan::OperationPlanStatus};

/// HTTP-side read model for an operation plan.
///
/// `created_at` is derived from the UUID v7 `id`; `updated_at` is the DB column.
#[derive(Debug, Clone)]
pub struct OperationPlanView {
    pub id: Uuid,
    pub module: ModuleKey,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub organization_id: Uuid,
}
