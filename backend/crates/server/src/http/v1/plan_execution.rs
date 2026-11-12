use std::sync::Arc;

use axum::extract::State;
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use domain::{
    Id,
    authorization::{Action, Caller, Permission},
    operation_plan::{OperationPlan, OperationPlanStatus as DomainStatus},
    planning::StopOutcome,
};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{
            dto::plan::{PlanResponse, StopOutcomeDto},
            error::ErrorBody,
            plan::{detail_response, plan_access},
            scope,
        },
    },
    service::{AuthError, ServiceError},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(start_plan))
        .routes(routes!(finish_plan))
        .routes(routes!(fail_plan))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct FinishPlanRequest {
    /// Exactly one outcome per stop of the plan.
    pub outcomes: Vec<StopOutcomeDto>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct FailPlanRequest {
    /// Why the tour ended early; must not be blank.
    pub note: String,
}

/// Executing a plan is open to whoever may edit it and to the people it is
/// assigned to, as long as they may read it.
pub(super) async fn authorize_execution(
    state: &AppState,
    caller: Caller,
    plan: &OperationPlan,
) -> Result<(), ServiceError> {
    let (access, resource) = plan_access(state, plan.module())?;
    let org = plan.organization_id();
    let ctx = state.authorization_service.context_for(caller).await?;
    scope::ensure_visible(&ctx, Permission::new(resource, Action::Read), org.value())?;
    let may_update = ctx.allows_in(Permission::new(resource, Action::Update), org);
    let assigned = plan.user_ids().contains(&caller.user_id);
    if !may_update && !assigned {
        return Err(AuthError::Forbidden.into());
    }
    access.ensure_active(org).await
}

pub fn same_outcomes(stored: &[StopOutcome], requested: &[StopOutcome]) -> bool {
    stored.len() == requested.len() && requested.iter().all(|r| stored.contains(r))
}

async fn respond(state: &AppState, id: Uuid) -> Result<Json<PlanResponse>, ServiceError> {
    let view = state.operation_plan_service.view_by_id(Id::new(id)).await?;
    Ok(Json(detail_response(state, view).await?))
}

#[utoipa::path(post, path = "/plans/{plan_id}/start", tag = "Plans",
    operation_id = "startPlan",
    summary = "Start a plan",
    description = "Planned to active. Allowed with the plan's update right, or for an assigned user with read right. Starting an active plan again changes nothing.",
    params(("plan_id" = Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Plan is active", body = PlanResponse),
        (status = 400, description = "Invalid status transition (code `plan.invalid_state_transition`)", body = ErrorBody),
        (status = 403, description = "Neither update right nor assigned, or the module is switched off (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn start_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<PlanResponse>, ServiceError> {
    let plan = state.operation_plan_service.by_id(Id::new(id)).await?;
    authorize_execution(&state, user.caller(), &plan).await?;
    if plan.status() != DomainStatus::Active {
        state.operation_plan_service.start(Id::new(id)).await?;
    }
    respond(&state, id).await
}

#[utoipa::path(post, path = "/plans/{plan_id}/finish", tag = "Plans",
    operation_id = "finishPlan",
    summary = "Finish a plan",
    description = "Active to finished with one outcome per stop. Repeating it with the stored outcomes changes nothing; with different ones it is a conflict.",
    params(("plan_id" = Uuid, Path, description = "Plan ID")),
    request_body = FinishPlanRequest,
    responses(
        (status = 200, description = "Plan is finished", body = PlanResponse),
        (status = 400, description = "Invalid transition or outcomes (codes `plan.*`)", body = ErrorBody),
        (status = 403, description = "Neither update right nor assigned, or the module is switched off (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 409, description = "Already finished with other outcomes (code `plan.already_finished_differently`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn finish_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
    Json(body): Json<FinishPlanRequest>,
) -> Result<Json<PlanResponse>, ServiceError> {
    let plan_id = Id::new(id);
    let plan = state.operation_plan_service.by_id(plan_id).await?;
    authorize_execution(&state, user.caller(), &plan).await?;
    let outcomes: Vec<StopOutcome> = body.outcomes.into_iter().map(Into::into).collect();
    if plan.status() == DomainStatus::Finished {
        let stored = state.operation_plan_service.outcomes(plan_id).await?;
        if !same_outcomes(&stored, &outcomes) {
            return Err(ServiceError::PlanFinishedDifferently);
        }
    } else {
        state
            .operation_execution_service
            .finish(plan_id, outcomes)
            .await?;
    }
    respond(&state, id).await
}

#[utoipa::path(post, path = "/plans/{plan_id}/fail", tag = "Plans",
    operation_id = "failPlan",
    summary = "End a plan early",
    description = "Active to not completed, with a note. Repeating it changes nothing.",
    params(("plan_id" = Uuid, Path, description = "Plan ID")),
    request_body = FailPlanRequest,
    responses(
        (status = 200, description = "Plan is not completed", body = PlanResponse),
        (status = 400, description = "Invalid transition or blank note (codes `plan.*`)", body = ErrorBody),
        (status = 403, description = "Neither update right nor assigned, or the module is switched off (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn fail_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
    Json(body): Json<FailPlanRequest>,
) -> Result<Json<PlanResponse>, ServiceError> {
    let plan = state.operation_plan_service.by_id(Id::new(id)).await?;
    authorize_execution(&state, user.caller(), &plan).await?;
    if plan.status() != DomainStatus::NotCompleted {
        state
            .operation_plan_service
            .fail(Id::new(id), body.note)
            .await?;
    }
    respond(&state, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::planning::{StopOutcome, StopRef};

    fn o(stop: StopRef, completed: bool, amount: Option<f64>) -> StopOutcome {
        StopOutcome {
            stop,
            completed,
            amount,
        }
    }

    #[test]
    fn same_outcomes_ignores_order() {
        let (a, b) = (
            StopRef::new(uuid::Uuid::now_v7()),
            StopRef::new(uuid::Uuid::now_v7()),
        );
        assert!(same_outcomes(
            &[o(a, true, Some(10.0)), o(b, false, None)],
            &[o(b, false, None), o(a, true, Some(10.0))],
        ));
    }

    #[test]
    fn a_different_amount_or_flag_is_not_the_same() {
        let a = StopRef::new(uuid::Uuid::now_v7());
        assert!(!same_outcomes(
            &[o(a, true, Some(10.0))],
            &[o(a, true, Some(11.0))]
        ));
        assert!(!same_outcomes(
            &[o(a, true, Some(10.0))],
            &[o(a, false, Some(10.0))]
        ));
    }

    #[test]
    fn a_missing_stop_is_not_the_same() {
        let (a, b) = (
            StopRef::new(uuid::Uuid::now_v7()),
            StopRef::new(uuid::Uuid::now_v7()),
        );
        assert!(!same_outcomes(
            &[o(a, true, None)],
            &[o(a, true, None), o(b, true, None)]
        ));
    }
}
