use std::{collections::HashMap, sync::Arc};

use axum::extract::State;
use chrono::{Duration, NaiveDate, Utc};
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use domain::{
    Id,
    authorization::{Action, Permission},
    operation_plan::{OperationPlanSearchQuery, OperationPlanView},
    shared::pagination::Pagination,
};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Query},
        v1::{
            dto::{
                plan::{PlanInListDetailView, PlanInListResponse, PlanStatus},
                vehicle::VehicleResponse,
            },
            error::ErrorBody,
            plan::plan_access,
        },
    },
    service::ServiceError,
};

const MAX_WINDOW_DAYS: i64 = 31;
const PAGE_SIZE: u64 = 100;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(list_my_plans))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct MyPlansParams {
    /// First day, inclusive (`YYYY-MM-DD`). Default: today.
    pub from: Option<NaiveDate>,
    /// Last day, inclusive. Default: `from` + 7 days. At most 31 days after `from`.
    pub to: Option<NaiveDate>,
    /// Repeatable. Default: `planned` and `active`.
    #[serde(default)]
    pub status: Vec<PlanStatus>,
}

#[utoipa::path(get, path = "/users/me/plans", tag = "Plans",
    operation_id = "listMyPlans",
    summary = "List the caller's assigned plans",
    description = "Plans of every planning module the caller is assigned to and may read, in modules active for the plan's organization, sorted by date ascending.",
    params(MyPlansParams),
    responses(
        (status = 200, description = "Assigned plans", body = Vec<PlanInListResponse>),
        (status = 400, description = "Window longer than 31 days or `to` before `from` (code `request.invalid_input`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_my_plans(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<MyPlansParams>,
) -> Result<Json<Vec<PlanInListResponse>>, ServiceError> {
    let from = params.from.unwrap_or_else(|| Utc::now().date_naive());
    let to = params.to.unwrap_or(from + Duration::days(7));
    if to < from || (to - from).num_days() > MAX_WINDOW_DAYS {
        return Err(ServiceError::InvalidInput(format!(
            "date window must be 0 to {MAX_WINDOW_DAYS} days"
        )));
    }
    let statuses = if params.status.is_empty() {
        vec![PlanStatus::Planned, PlanStatus::Active]
    } else {
        params.status
    };
    let caller = user.caller();

    let mut views: Vec<OperationPlanView> = Vec::new();
    for module in state.planning.modules() {
        let (access, resource) = plan_access(&state, &module)?;
        let visible = access
            .visible_orgs(caller, Permission::new(resource, Action::Read))
            .await?;
        let mut page_no = 1;
        loop {
            let query = OperationPlanSearchQuery {
                module: module.clone(),
                provider: None,
                statuses: statuses.iter().copied().map(Into::into).collect(),
                visible: visible.clone(),
                assigned_user: Some(caller.user_id),
                date_from: Some(from),
                date_to: Some(to + Duration::days(1)),
            };
            let page = state
                .operation_plan_service
                .search_view(query, Pagination::new(page_no, PAGE_SIZE))
                .await?;
            let fetched = page.items.len() as u64;
            views.extend(page.items);
            if fetched < PAGE_SIZE || page_no * PAGE_SIZE >= page.total {
                break;
            }
            page_no += 1;
        }
    }
    views.sort_by_key(|v| v.date);

    let vehicle_ids: Vec<_> = views
        .iter()
        .flat_map(|v| {
            [v.transporter_id, v.trailer_id]
                .into_iter()
                .flatten()
                .map(Id::new)
        })
        .collect();
    let vehicles = state.vehicle_service.view_by_ids(&vehicle_ids).await?;
    let vehicle_map: HashMap<_, _> = vehicles.iter().map(|v| (v.id, v)).collect();

    let response = views
        .into_iter()
        .filter_map(|view| {
            let transporter = view
                .transporter_id
                .and_then(|id| vehicle_map.get(&id))
                .map(|v| VehicleResponse::from(*v))?;
            let trailer = view
                .trailer_id
                .and_then(|id| vehicle_map.get(&id))
                .map(|v| VehicleResponse::from(*v));
            Some(PlanInListResponse::from(PlanInListDetailView {
                view,
                transporter,
                trailer,
            }))
        })
        .collect();
    Ok(Json(response))
}
