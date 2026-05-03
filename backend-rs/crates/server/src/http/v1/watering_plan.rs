use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                cluster::TreeClusterInListResponse,
                vehicle::VehicleResponse,
                watering_plan::{
                    EvaluationValueResponse, WateringPlanCreateRequest, WateringPlanDetailView,
                    WateringPlanInListDetailView, WateringPlanInListResponse, WateringPlanResponse,
                    WateringPlanUpdateRequest,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    shared::{
        pagination::Pagination,
        provenance::{Provenance, ProviderId},
    },
    watering_plan::{
        WateringPlanSearchQuery, WateringPlanStatus as DomainStatus, WateringPlanUpdate,
        WateringPlanView,
    },
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_watering_plans, create_watering_plan))
        .routes(routes!(get_gpx_file))
        .routes(routes!(preview_route))
        .routes(routes!(
            get_watering_plan,
            update_watering_plan,
            delete_watering_plan
        ))
}

async fn resolve_view_relations(
    state: &AppState,
    view: &WateringPlanView,
) -> Result<
    (
        VehicleResponse,
        Option<VehicleResponse>,
        Vec<TreeClusterInListResponse>,
        Vec<EvaluationValueResponse>,
    ),
    ServiceError,
> {
    let transporter_id = view
        .transporter_id
        .map(Id::new)
        .ok_or_else(|| ServiceError::InvalidInput("watering plan has no transporter".into()))?;
    let transporter =
        VehicleResponse::from(&state.vehicle_service.view_by_id(transporter_id).await?);

    let trailer = match view.trailer_id {
        Some(id) => Some(VehicleResponse::from(
            &state.vehicle_service.view_by_id(Id::new(id)).await?,
        )),
        None => None,
    };

    let cluster_ids: Vec<_> = view.cluster_ids.iter().copied().map(Id::new).collect();
    let clusters = state.cluster_service.by_ids(&cluster_ids).await?;
    let region_ids: Vec<_> = clusters.iter().filter_map(|c| c.region_id()).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();

    let cluster_responses: Vec<TreeClusterInListResponse> = clusters
        .iter()
        .map(|c| {
            let region = c.region_id().and_then(|id| region_map.get(&id).copied());
            TreeClusterInListResponse::from((c, region))
        })
        .collect();

    let plan_id = Id::new(view.id);
    let evals = state.watering_plan_service.evaluations(plan_id).await?;
    let evaluation_responses: Vec<EvaluationValueResponse> = evals
        .into_iter()
        .map(|e| EvaluationValueResponse {
            watering_plan_id: e.watering_plan_id.value(),
            tree_cluster_id: e.cluster_id.value(),
            consumed_water: e.consumed_water,
        })
        .collect();

    Ok((
        transporter,
        trailer,
        cluster_responses,
        evaluation_responses,
    ))
}

#[utoipa::path(get, path = "/watering-plans", tag = "Watering Plans",
    operation_id = "listWateringPlans",
    summary = "List all watering plans",
    description = "Returns a paginated list of watering plans with embedded vehicles and clusters.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of watering plans", body = ListResponse<WateringPlanInListResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_watering_plans(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<WateringPlanInListResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .watering_plan_service
        .search_view(WateringPlanSearchQuery::default(), pagination)
        .await?;

    let vehicle_ids: Vec<_> = page
        .items
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

    let cluster_ids: Vec<_> = page
        .items
        .iter()
        .flat_map(|v| v.cluster_ids.iter().copied().map(Id::new))
        .collect();
    let clusters = state.cluster_service.by_ids(&cluster_ids).await?;
    let region_ids: Vec<_> = clusters.iter().filter_map(|c| c.region_id()).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();
    let cluster_map: HashMap<_, _> = clusters.iter().map(|c| (c.id.value(), c)).collect();

    let response = ListResponse::from_page_with(page, &pagination, |view: &WateringPlanView| {
        let transporter = view
            .transporter_id
            .and_then(|id| vehicle_map.get(&id))
            .map(|v| VehicleResponse::from(*v))
            .expect("watering plan must have a transporter");
        let trailer = view
            .trailer_id
            .and_then(|id| vehicle_map.get(&id))
            .map(|v| VehicleResponse::from(*v));
        let plan_clusters: Vec<TreeClusterInListResponse> = view
            .cluster_ids
            .iter()
            .filter_map(|cid| cluster_map.get(cid))
            .map(|c| {
                let region = c.region_id().and_then(|rid| region_map.get(&rid).copied());
                TreeClusterInListResponse::from((*c, region))
            })
            .collect();

        WateringPlanInListResponse::from(WateringPlanInListDetailView {
            view: view.clone(),
            transporter,
            trailer,
            clusters: plan_clusters,
            user_ids: vec![],
        })
    });

    Ok(Json(response))
}

#[utoipa::path(get, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "getWateringPlan",
    summary = "Get a watering plan",
    description = "Returns full watering plan detail including evaluation values.",
    params(("watering_plan_id" = i32, Path, description = "Watering plan ID")),
    responses(
        (status = 200, description = "Watering plan found", body = WateringPlanResponse),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = id))]
pub async fn get_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let view = state.watering_plan_service.view_by_id(Id::from(id)).await?;
    let (transporter, trailer, clusters, evaluation) =
        resolve_view_relations(&state, &view).await?;

    let response = WateringPlanResponse::from(WateringPlanDetailView {
        view,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation,
    });

    Ok(Json(response))
}

#[utoipa::path(post, path = "/watering-plans", tag = "Watering Plans",
    operation_id = "createWateringPlan",
    summary = "Create a watering plan",
    description = "Creates a new watering plan with clusters, vehicles, and user assignments.",
    request_body = WateringPlanCreateRequest,
    responses(
        (status = 201, description = "Watering plan created", body = WateringPlanResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_watering_plan(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<WateringPlanCreateRequest>,
) -> Result<(StatusCode, Json<WateringPlanResponse>), ServiceError> {
    let draft = entity.try_into()?;
    let plan = state.watering_plan_service.create(draft).await?;
    let view = state.watering_plan_service.view_by_id(plan.id).await?;
    let (transporter, trailer, clusters, evaluation) =
        resolve_view_relations(&state, &view).await?;

    let response = WateringPlanResponse::from(WateringPlanDetailView {
        view,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation,
    });

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(put, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "updateWateringPlan",
    summary = "Update a watering plan",
    description = "Updates the details or status of an existing watering plan.",
    params(("watering_plan_id" = i32, Path, description = "Watering plan ID")),
    request_body = WateringPlanUpdateRequest,
    responses(
        (status = 200, description = "Watering plan updated", body = WateringPlanResponse),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = id))]
pub async fn update_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<WateringPlanUpdateRequest>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let plan_id = Id::from(id);
    let current = state.watering_plan_service.by_id(plan_id).await?;
    let new_status: DomainStatus = entity.status.into();

    if current.status() == DomainStatus::Planned {
        let update = WateringPlanUpdate {
            date: parse_date(&entity.date)?,
            description: if entity.description.is_empty() {
                None
            } else {
                Some(entity.description.clone())
            },
            cluster_ids: entity
                .tree_cluster_ids
                .iter()
                .copied()
                .map(Id::new)
                .collect(),
            transporter_id: Some(Id::new(entity.transporter_id)),
            trailer_id: entity.trailer_id.map(Id::new),
            provenance: Provenance::new(
                entity
                    .provider
                    .clone()
                    .map(ProviderId::new)
                    .transpose()
                    .map_err(invalid)?,
                entity.additional_information.clone(),
            ),
        };
        state
            .watering_plan_service
            .replace_details(plan_id, update)
            .await?;
    }

    match (current.status(), new_status) {
        (a, b) if a == b => {}
        (DomainStatus::Planned, DomainStatus::Active) => {
            state.watering_plan_service.start(plan_id).await?;
        }
        (DomainStatus::Planned | DomainStatus::Active, DomainStatus::Canceled) => {
            if entity.cancellation_note.trim().is_empty() {
                return Err(ServiceError::InvalidInput(
                    "cancellation_note required".into(),
                ));
            }
            state
                .watering_plan_service
                .cancel(plan_id, entity.cancellation_note)
                .await?;
        }
        (DomainStatus::Active, DomainStatus::NotCompleted) => {
            if entity.cancellation_note.trim().is_empty() {
                return Err(ServiceError::InvalidInput(
                    "cancellation_note required".into(),
                ));
            }
            state
                .watering_plan_service
                .fail(plan_id, entity.cancellation_note)
                .await?;
        }
        (DomainStatus::Active, DomainStatus::Finished) => {
            let evaluations: Vec<_> = entity
                .evaluation
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.into_domain(plan_id))
                .collect();
            state
                .watering_execution_service
                .finish(plan_id, evaluations)
                .await?;
        }
        (from, to) => {
            return Err(ServiceError::InvalidInput(format!(
                "invalid status transition from {from:?} to {to:?}"
            )));
        }
    }

    let view = state.watering_plan_service.view_by_id(plan_id).await?;
    let (transporter, trailer, clusters, evaluation) =
        resolve_view_relations(&state, &view).await?;

    let response = WateringPlanResponse::from(WateringPlanDetailView {
        view,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation,
    });

    Ok(Json(response))
}

#[utoipa::path(delete, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "deleteWateringPlan",
    summary = "Delete a watering plan",
    description = "Permanently deletes a watering plan.",
    params(("watering_plan_id" = i32, Path, description = "Watering plan ID")),
    responses(
        (status = 204, description = "Watering plan deleted"),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = id))]
pub async fn delete_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.watering_plan_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/watering-plans/route/gpx/{gpx_name}", tag = "Watering Plans",
    operation_id = "getGpxFile",
    summary = "Download GPX file",
    description = "Downloads the optimized watering route as a GPX file.",
    params(("gpx_name" = String, Path, description = "GPX file name")),
    responses(
        (status = 200, description = "GPX file"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_gpx_file(
    State(_state): State<Arc<AppState>>,
    Path(_name): Path<String>,
) -> Result<Json<()>, ServiceError> {
    todo!()
}

#[utoipa::path(post, path = "/watering-plans/route/preview", tag = "Watering Plans",
    operation_id = "previewRoute",
    summary = "Preview route",
    description = "Calculates and previews an optimized watering route without creating a plan.",
    responses(
        (status = 200, description = "Route preview"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn preview_route(State(_state): State<Arc<AppState>>) -> Result<Json<()>, ServiceError> {
    todo!()
}

fn parse_date(s: &str) -> Result<chrono::DateTime<chrono::Utc>, ServiceError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| ServiceError::InvalidInput(format!("invalid date: {e}")))
}

fn invalid(e: domain::shared::error::ValidationError) -> ServiceError {
    ServiceError::InvalidInput(e.to_string())
}
