use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::Query;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                cluster::TreeClusterInListResponse,
                vehicle::VehicleResponse,
                watering_plan::{
                    EvaluationValueResponse, RouteGeometry, RouteRequest, RouteResponse,
                    WateringPlanCreateRequest, WateringPlanDetailView,
                    WateringPlanInListDetailView, WateringPlanInListResponse,
                    WateringPlanListParams, WateringPlanResponse, WateringPlanUpdateRequest,
                    parse_user_ids,
                },
            },
            gpx, scope,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
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
        .routes(routes!(preview_route))
        .routes(routes!(get_watering_plan_route))
        .routes(routes!(get_gpx_file))
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
    description = "Returns a paginated list of watering plans with embedded vehicles and clusters. Optional filter parameter (status) narrows the result; the array parameter is repeatable.",
    params(WateringPlanListParams),
    responses(
        (status = 200, description = "Paginated list of watering plans", body = ListResponse<WateringPlanInListResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_watering_plans(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<WateringPlanListParams>,
) -> Result<Json<ListResponse<WateringPlanInListResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let visible = state
        .authorization_service
        .visible_orgs_for(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Read),
        )
        .await?;
    let query = WateringPlanSearchQuery {
        statuses: params.status.into_iter().map(Into::into).collect(),
        visible,
        ..Default::default()
    };
    let page = state
        .watering_plan_service
        .search_view(query, pagination)
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

    let response =
        ListResponse::from_page_filter_map(page, &pagination, |view: &WateringPlanView| {
            let Some(transporter) = view
                .transporter_id
                .and_then(|id| vehicle_map.get(&id))
                .map(|v| VehicleResponse::from(*v))
            else {
                tracing::warn!(
                    plan.id = %view.id,
                    "skipping watering plan without resolvable transporter"
                );
                return None;
            };
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

            Some(WateringPlanInListResponse::from(
                WateringPlanInListDetailView {
                    view: view.clone(),
                    transporter,
                    trailer,
                    clusters: plan_clusters,
                    user_ids: view.user_ids.iter().map(|u| u.to_string()).collect(),
                },
            ))
        });

    Ok(Json(response))
}

#[utoipa::path(get, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "getWateringPlan",
    summary = "Get a watering plan",
    description = "Returns full watering plan detail including evaluation values.",
    params(("watering_plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    responses(
        (status = 200, description = "Watering plan found", body = WateringPlanResponse),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_watering_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let view = state.watering_plan_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        &scope::effective_orgs(view.organization_id, &[]),
    )?;
    let (transporter, trailer, clusters, evaluation) =
        resolve_view_relations(&state, &view).await?;

    let response = WateringPlanResponse::from(WateringPlanDetailView {
        user_ids: view.user_ids.iter().map(|u| u.to_string()).collect(),
        view,
        transporter,
        trailer,
        clusters,
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
    user: AuthUserExtractor,
    Json(entity): Json<WateringPlanCreateRequest>,
) -> Result<(StatusCode, Json<WateringPlanResponse>), ServiceError> {
    let org = scope::resolve_target_org(&state, user.id, entity.organization_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Create),
            org,
        )
        .await?;

    let ctx = state.authorization_service.context_for(user.id).await?;
    let transporter = state
        .vehicle_service
        .view_by_id(Id::new(entity.transporter_id))
        .await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &scope::effective_orgs(transporter.organization_id, &[]),
    )?;
    if let Some(trailer_id) = entity.trailer_id {
        let trailer = state
            .vehicle_service
            .view_by_id(Id::new(trailer_id))
            .await?;
        scope::ensure_visible(
            &ctx,
            Permission::new(Resource::Vehicle, Action::Read),
            &scope::effective_orgs(trailer.organization_id, &[]),
        )?;
    }

    let draft = entity.into_draft(org)?;
    let plan = state.watering_plan_service.create(draft).await?;
    let view = state.watering_plan_service.view_by_id(plan.id).await?;
    let (transporter, trailer, clusters, evaluation) =
        resolve_view_relations(&state, &view).await?;

    let response = WateringPlanResponse::from(WateringPlanDetailView {
        user_ids: view.user_ids.iter().map(|u| u.to_string()).collect(),
        view,
        transporter,
        trailer,
        clusters,
        evaluation,
    });

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(put, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "updateWateringPlan",
    summary = "Update a watering plan",
    description = "Updates the details or status of an existing watering plan.",
    params(("watering_plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    request_body = WateringPlanUpdateRequest,
    responses(
        (status = 200, description = "Watering plan updated", body = WateringPlanResponse),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn update_watering_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<WateringPlanUpdateRequest>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let plan_id = Id::new(id);
    let current = state.watering_plan_service.by_id(plan_id).await?;
    let new_status: DomainStatus = entity.status.into();

    let ctx = state.authorization_service.context_for(user.id).await?;
    let plan_orgs = scope::effective_orgs(current.organization_id().value(), &[]);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        &plan_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Update),
            &plan_orgs,
        )
        .await?;

    if current.status() == DomainStatus::Planned {
        let transporter = state
            .vehicle_service
            .view_by_id(Id::new(entity.transporter_id))
            .await?;
        scope::ensure_visible(
            &ctx,
            Permission::new(Resource::Vehicle, Action::Read),
            &scope::effective_orgs(transporter.organization_id, &[]),
        )?;
        if let Some(trailer_id) = entity.trailer_id {
            let trailer = state
                .vehicle_service
                .view_by_id(Id::new(trailer_id))
                .await?;
            scope::ensure_visible(
                &ctx,
                Permission::new(Resource::Vehicle, Action::Read),
                &scope::effective_orgs(trailer.organization_id, &[]),
            )?;
        }

        let update = WateringPlanUpdate {
            date: parse_date(&entity.date)?,
            description: if entity.description.is_empty() {
                None
            } else {
                Some(entity.description.clone())
            },
            start_point_name: entity.start_point_name.clone(),
            cluster_ids: entity
                .tree_cluster_ids
                .iter()
                .copied()
                .map(Id::new)
                .collect(),
            transporter_id: Some(Id::new(entity.transporter_id)),
            trailer_id: entity.trailer_id.map(Id::new),
            provenance: Provenance::new(
                entity.provider.clone().map(ProviderId::new).transpose()?,
                entity.additional_information.clone(),
            ),
            user_ids: parse_user_ids(&entity.user_ids)?,
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
        (DomainStatus::Active, DomainStatus::Planned) => {
            state.watering_plan_service.revert_start(plan_id).await?;
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
        user_ids: view.user_ids.iter().map(|u| u.to_string()).collect(),
        view,
        transporter,
        trailer,
        clusters,
        evaluation,
    });

    Ok(Json(response))
}

#[utoipa::path(delete, path = "/watering-plans/{watering_plan_id}", tag = "Watering Plans",
    operation_id = "deleteWateringPlan",
    summary = "Delete a watering plan",
    description = "Permanently deletes a watering plan.",
    params(("watering_plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    responses(
        (status = 204, description = "Watering plan deleted"),
        (status = 404, description = "Watering plan not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn delete_watering_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let plan_id = Id::new(id);
    let current = state.watering_plan_service.by_id(plan_id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        &scope::effective_orgs(current.organization_id().value(), &[]),
    )?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Delete),
            current.organization_id(),
        )
        .await?;
    state.watering_plan_service.delete(plan_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn route_response_from_plan(
    plan: &domain::watering_plan::WateringPlan,
) -> Result<RouteResponse, ServiceError> {
    let geometry = plan
        .route_geometry()
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    Ok(RouteResponse {
        distance: plan.distance.map(|d| d.meters()).unwrap_or_default(),
        duration: plan.duration.as_secs_f64(),
        refill_count: plan.refill_count,
        refill_points: plan.refill_points().iter().map(Into::into).collect(),
        total_water_required: plan.total_water_required.unwrap_or_default(),
        geometry: RouteGeometry::from_coordinates(geometry),
    })
}

#[utoipa::path(get, path = "/watering-plans/{watering_plan_id}/route", tag = "Watering Plans",
    operation_id = "getWateringPlanRoute",
    summary = "Get the optimized route of a watering plan",
    description = "Returns the persisted optimized route as GeoJSON LineString geometry with metrics.",
    params(("watering_plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    responses(
        (status = 200, description = "Optimized route", body = RouteResponse),
        (status = 404, description = "Plan not found or has no route"),
        (status = 503, description = "Routing feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_watering_plan_route(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<RouteResponse>, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "routing" });
    }
    let view = state.watering_plan_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        &scope::effective_orgs(view.organization_id, &[]),
    )?;
    let plan = state.watering_plan_service.by_id(Id::new(id)).await?;
    Ok(Json(route_response_from_plan(&plan)?))
}

#[utoipa::path(post, path = "/watering-plans/route/preview", tag = "Watering Plans",
    operation_id = "previewRoute",
    summary = "Preview route",
    description = "Calculates and previews an optimized watering route without creating a plan.",
    request_body = RouteRequest,
    responses(
        (status = 200, description = "Route preview", body = RouteResponse),
        (status = 422, description = "Route problem rejected by the optimizer"),
        (status = 502, description = "Routing engine unavailable"),
        (status = 503, description = "Routing feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn preview_route(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<RouteRequest>,
) -> Result<Json<RouteResponse>, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "routing" });
    }
    let org = scope::resolve_target_org(&state, user.id, None).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Create),
            org,
        )
        .await?;
    let computed = state
        .watering_plan_service
        .preview_route(
            req.cluster_ids.into_iter().map(Id::new).collect(),
            Id::new(req.transporter_id),
            req.trailer_id.map(Id::new),
            req.start_point_name,
            org,
        )
        .await?;
    Ok(Json(RouteResponse {
        distance: computed.route.distance.meters(),
        duration: computed.route.duration.as_secs_f64(),
        refill_count: computed.route.refill_count,
        refill_points: computed.refill_points.iter().map(Into::into).collect(),
        total_water_required: computed.total_water_liters,
        geometry: RouteGeometry::from_coordinates(&computed.route.geometry),
    }))
}

#[utoipa::path(get, path = "/watering-plans/{watering_plan_id}/route/gpx", tag = "Watering Plans",
    operation_id = "getGpxFile",
    summary = "Download GPX file",
    description = "Renders the optimized watering route of a plan as a GPX track.",
    params(("watering_plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    responses(
        (status = 200, description = "GPX file", content_type = "application/gpx+xml"),
        (status = 404, description = "Plan not found or has no route"),
        (status = 503, description = "Routing feature is disabled"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_gpx_file(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "routing" });
    }
    let view = state.watering_plan_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        &scope::effective_orgs(view.organization_id, &[]),
    )?;
    let plan = state.watering_plan_service.by_id(Id::new(id)).await?;
    let geometry = plan
        .route_geometry()
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    let name = format!("Bewässerungsroute {}", plan.date.format("%Y-%m-%d"));
    let body = gpx::render_gpx(&name, geometry);
    Ok((
        [
            (header::CONTENT_TYPE, "application/gpx+xml".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=route-{}.gpx",
                    plan.date.format("%Y-%m-%d")
                ),
            ),
        ],
        body,
    ))
}

fn parse_date(s: &str) -> Result<chrono::DateTime<chrono::Utc>, ServiceError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| ServiceError::InvalidInput(format!("invalid date: {e}")))
}
