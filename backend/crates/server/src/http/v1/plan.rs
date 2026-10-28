use std::{collections::HashMap, sync::Arc};

use crate::http::v1::error::ErrorBody;

use crate::http::extractors::{Json, Path, Query};
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                plan::{
                    PlanCreateRequest, PlanDetailView, PlanInListDetailView, PlanInListResponse,
                    PlanListParams, PlanResponse, PlanRouteRequest, PlanUpdateRequest,
                    RouteGeometry, RouteResponse, StopOutcomeDto, parse_date, parse_user_ids,
                    stop_refs,
                },
                vehicle::VehicleResponse,
            },
            gpx, scope,
        },
    },
    service::{
        Feature, ServiceError, module_access::ModuleAccess, operation_plan_service::PreviewRoute,
    },
};
use domain::{
    Id,
    authorization::{Action, Caller, Permission, ResourceKey},
    module::ModuleKey,
    operation_plan::{
        OperationPlan, OperationPlanSearchQuery, OperationPlanStatus as DomainStatus,
        OperationPlanUpdate, OperationPlanView,
    },
    shared::{
        pagination::Pagination,
        provenance::{Provenance, ProviderId},
    },
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_plans, create_plan))
        .routes(routes!(preview_plan_route))
        .routes(routes!(get_plan_route))
        .routes(routes!(get_plan_gpx))
        .routes(routes!(get_plan, update_plan, delete_plan))
}

/// Permissions on a plan hang on the plan resource its module declares, and
/// writes additionally need the module to be active in the organization.
fn plan_access(
    state: &AppState,
    module: &ModuleKey,
) -> Result<(ModuleAccess, ResourceKey), ServiceError> {
    let resource = state.planning.provider(module)?.plan_resource();
    let access = ModuleAccess::new(
        module.clone(),
        state.authorization_service.clone(),
        state.module_activation.clone(),
    );
    Ok((access, resource))
}

fn plan_resource(state: &AppState, module: &ModuleKey) -> Result<ResourceKey, ServiceError> {
    Ok(state.planning.provider(module)?.plan_resource())
}

async fn ensure_vehicles_visible(
    state: &AppState,
    caller: Caller,
    transporter_id: uuid::Uuid,
    trailer_id: Option<uuid::Uuid>,
) -> Result<(), ServiceError> {
    let ctx = state.authorization_service.context_for(caller).await?;
    let read = Permission::new(ResourceKey::VEHICLE, Action::Read);
    let transporter = state
        .vehicle_service
        .view_by_id(Id::new(transporter_id))
        .await?;
    scope::ensure_visible(&ctx, read, transporter.organization_id)?;
    if let Some(trailer_id) = trailer_id {
        let trailer = state
            .vehicle_service
            .view_by_id(Id::new(trailer_id))
            .await?;
        scope::ensure_visible(&ctx, read, trailer.organization_id)?;
    }
    Ok(())
}

async fn detail_response(
    state: &AppState,
    view: OperationPlanView,
) -> Result<PlanResponse, ServiceError> {
    let transporter_id = view
        .transporter_id
        .map(Id::new)
        .ok_or_else(|| ServiceError::InvalidInput("plan has no transporter".into()))?;
    let transporter =
        VehicleResponse::from(&state.vehicle_service.view_by_id(transporter_id).await?);

    let trailer = match view.trailer_id {
        Some(id) => Some(VehicleResponse::from(
            &state.vehicle_service.view_by_id(Id::new(id)).await?,
        )),
        None => None,
    };

    let outcomes: Vec<StopOutcomeDto> = state
        .operation_plan_service
        .outcomes(Id::new(view.id))
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(PlanResponse::from(PlanDetailView {
        view,
        transporter,
        trailer,
        outcomes,
    }))
}

#[utoipa::path(get, path = "/plans", tag = "Plans",
    operation_id = "listPlans",
    summary = "List the plans of a module",
    description = "Returns a paginated list of one module's plans with embedded vehicles. Plans of organizations in which the module is switched off are left out. Optional filter parameter (status) narrows the result; the array parameter is repeatable.",
    params(PlanListParams),
    responses(
        (status = 200, description = "Paginated list of plans", body = ListResponse<PlanInListResponse>),
        (status = 400, description = "Missing or malformed module", body = ErrorBody),
        (status = 422, description = "No module with this key plans tours (code `module.unknown`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(module = %params.module))]
pub async fn list_plans(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<PlanListParams>,
) -> Result<Json<ListResponse<PlanInListResponse>>, ServiceError> {
    let module = ModuleKey::new(&params.module)?;
    let (access, resource) = plan_access(&state, &module)?;
    let pagination = Pagination::new(params.page, params.per_page);
    let visible = access
        .visible_orgs(user.caller(), Permission::new(resource, Action::Read))
        .await?;
    let query = OperationPlanSearchQuery {
        module,
        provider: None,
        statuses: params.status.into_iter().map(Into::into).collect(),
        visible,
    };
    let page = state
        .operation_plan_service
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

    let response =
        ListResponse::from_page_filter_map(page, &pagination, |view: &OperationPlanView| {
            let Some(transporter) = view
                .transporter_id
                .and_then(|id| vehicle_map.get(&id))
                .map(|v| VehicleResponse::from(*v))
            else {
                tracing::warn!(
                    plan.id = %view.id,
                    "skipping plan without resolvable transporter"
                );
                return None;
            };
            let trailer = view
                .trailer_id
                .and_then(|id| vehicle_map.get(&id))
                .map(|v| VehicleResponse::from(*v));
            Some(PlanInListResponse::from(PlanInListDetailView {
                view: view.clone(),
                transporter,
                trailer,
            }))
        });

    Ok(Json(response))
}

#[utoipa::path(get, path = "/plans/{plan_id}", tag = "Plans",
    operation_id = "getPlan",
    summary = "Get a plan",
    description = "Returns full plan detail including the per-stop outcomes.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Plan found", body = PlanResponse),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<PlanResponse>, ServiceError> {
    let view = state.operation_plan_service.view_by_id(Id::new(id)).await?;
    let resource = plan_resource(&state, &view.module)?;
    let ctx = state
        .authorization_service
        .context_for(user.caller())
        .await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resource, Action::Read),
        view.organization_id,
    )?;
    Ok(Json(detail_response(&state, view).await?))
}

#[utoipa::path(post, path = "/plans", tag = "Plans",
    operation_id = "createPlan",
    summary = "Create a plan",
    description = "Creates a new plan of a module with its stops, vehicles, and user assignments.",
    request_body = PlanCreateRequest,
    responses(
        (status = 201, description = "Plan created", body = PlanResponse),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 403, description = "Missing permission, or the module is switched off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 422, description = "A stop is unknown (code `plan.unknown_stop`) or outside the plan's organization (code `organization_mismatch.stops_vs_plan`), a vehicle lacks the module's capacity kind (code `vehicle.capacity_missing`), or the module is unknown (code `module.unknown`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(module = %entity.module))]
pub async fn create_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(entity): Json<PlanCreateRequest>,
) -> Result<(StatusCode, Json<PlanResponse>), ServiceError> {
    let module = ModuleKey::new(&entity.module)?;
    let (access, resource) = plan_access(&state, &module)?;
    let org = scope::resolve_target_org(
        &state.user_service,
        &state.authorization_service,
        user.caller(),
        entity.organization_id,
    )
    .await?;
    access
        .require_write(
            user.caller(),
            Permission::new(resource, Action::Create),
            org,
        )
        .await?;
    ensure_vehicles_visible(
        &state,
        user.caller(),
        entity.transporter_id,
        entity.trailer_id,
    )
    .await?;

    let draft = entity.into_draft(module, org)?;
    let plan = state.operation_plan_service.create(draft).await?;
    let view = state.operation_plan_service.view_by_id(plan.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(detail_response(&state, view).await?),
    ))
}

#[utoipa::path(put, path = "/plans/{plan_id}", tag = "Plans",
    operation_id = "updatePlan",
    summary = "Update a plan",
    description = "Updates the details or status of an existing plan. Details only change while the plan is planned; repeating the current status changes nothing.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    request_body = PlanUpdateRequest,
    responses(
        (status = 200, description = "Plan updated", body = PlanResponse),
        (status = 400, description = "Invalid input or status transition (codes `plan.*`)", body = ErrorBody),
        (status = 403, description = "Missing permission, or the module is switched off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 422, description = "A stop is unknown (code `plan.unknown_stop`) or outside the plan's organization (code `organization_mismatch.stops_vs_plan`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn update_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<PlanUpdateRequest>,
) -> Result<Json<PlanResponse>, ServiceError> {
    let plan_id = Id::new(id);
    let current = state.operation_plan_service.by_id(plan_id).await?;
    let new_status: DomainStatus = entity.status.into();
    let (access, resource) = plan_access(&state, current.module())?;

    let ctx = state
        .authorization_service
        .context_for(user.caller())
        .await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resource, Action::Read),
        current.organization_id().value(),
    )?;
    access
        .require_write(
            user.caller(),
            Permission::new(resource, Action::Update),
            current.organization_id(),
        )
        .await?;

    if current.status() == DomainStatus::Planned {
        ensure_vehicles_visible(
            &state,
            user.caller(),
            entity.transporter_id,
            entity.trailer_id,
        )
        .await?;

        let update = OperationPlanUpdate {
            date: parse_date(&entity.date)?,
            description: if entity.description.is_empty() {
                None
            } else {
                Some(entity.description.clone())
            },
            start_point_name: entity.start_point_name.clone(),
            stops: stop_refs(&entity.stops),
            transporter_id: Some(Id::new(entity.transporter_id)),
            trailer_id: entity.trailer_id.map(Id::new),
            transporter_capacity_used: entity.transporter_capacity_used,
            provenance: Provenance::new(
                entity.provider.clone().map(ProviderId::new).transpose()?,
                entity.additional_information.clone(),
            ),
            user_ids: parse_user_ids(&entity.user_ids)?,
        };
        state
            .operation_plan_service
            .replace_details(plan_id, update)
            .await?;
    }

    match (current.status(), new_status) {
        (a, b) if a == b => {}
        (DomainStatus::Planned, DomainStatus::Active) => {
            state.operation_plan_service.start(plan_id).await?;
        }
        (DomainStatus::Active, DomainStatus::Planned) => {
            state.operation_plan_service.revert_start(plan_id).await?;
        }
        (DomainStatus::Planned | DomainStatus::Active, DomainStatus::Canceled) => {
            if entity.cancellation_note.trim().is_empty() {
                return Err(ServiceError::InvalidInput(
                    "cancellation_note required".into(),
                ));
            }
            state
                .operation_plan_service
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
                .operation_plan_service
                .fail(plan_id, entity.cancellation_note)
                .await?;
        }
        (DomainStatus::Active, DomainStatus::Finished) => {
            let outcomes = entity
                .outcomes
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect();
            state
                .operation_execution_service
                .finish(plan_id, outcomes)
                .await?;
        }
        (from, to) => {
            return Err(ServiceError::InvalidInput(format!(
                "invalid status transition from {from:?} to {to:?}"
            )));
        }
    }

    let view = state.operation_plan_service.view_by_id(plan_id).await?;
    Ok(Json(detail_response(&state, view).await?))
}

#[utoipa::path(delete, path = "/plans/{plan_id}", tag = "Plans",
    operation_id = "deletePlan",
    summary = "Delete a plan",
    description = "Permanently deletes a plan.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    responses(
        (status = 204, description = "Plan deleted"),
        (status = 403, description = "Missing permission, or the module is switched off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn delete_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let plan_id = Id::new(id);
    let current = state.operation_plan_service.by_id(plan_id).await?;
    let (access, resource) = plan_access(&state, current.module())?;
    let ctx = state
        .authorization_service
        .context_for(user.caller())
        .await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resource, Action::Read),
        current.organization_id().value(),
    )?;
    access
        .require_write(
            user.caller(),
            Permission::new(resource, Action::Delete),
            current.organization_id(),
        )
        .await?;
    state.operation_plan_service.delete(plan_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn route_response_from_plan(plan: &OperationPlan) -> Result<RouteResponse, ServiceError> {
    let geometry = plan
        .route_geometry()
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    Ok(RouteResponse {
        distance: plan.distance.map(|d| d.meters()).unwrap_or_default(),
        duration: plan.duration.as_secs_f64(),
        refill_count: plan.refill_count,
        station_visits: plan.station_visits().iter().map(Into::into).collect(),
        total_demand: plan.total_demand.unwrap_or_default(),
        geometry: RouteGeometry::from_coordinates(geometry),
    })
}

/// Loads a plan after checking the routing flag and the caller's read right.
async fn readable_plan(
    state: &AppState,
    caller: Caller,
    id: uuid::Uuid,
) -> Result<OperationPlan, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled {
            feature: Feature::Routing,
        });
    }
    let plan = state.operation_plan_service.by_id(Id::new(id)).await?;
    let resource = plan_resource(state, plan.module())?;
    let ctx = state.authorization_service.context_for(caller).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resource, Action::Read),
        plan.organization_id().value(),
    )?;
    Ok(plan)
}

#[utoipa::path(get, path = "/plans/{plan_id}/route", tag = "Plans",
    operation_id = "getPlanRoute",
    summary = "Get the optimized route of a plan",
    description = "Returns the persisted optimized route as GeoJSON LineString geometry with metrics.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Optimized route", body = RouteResponse),
        (status = 404, description = "Plan not found or has no route", body = ErrorBody),
        (status = 503, description = "Routing feature is disabled (code `feature.routing_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_plan_route(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<RouteResponse>, ServiceError> {
    let plan = readable_plan(&state, user.caller(), id).await?;
    Ok(Json(route_response_from_plan(&plan)?))
}

#[utoipa::path(post, path = "/plans/route/preview", tag = "Plans",
    operation_id = "previewPlanRoute",
    summary = "Preview a route",
    description = "Calculates and previews an optimized route over a module's stops without creating a plan.",
    request_body = PlanRouteRequest,
    responses(
        (status = 200, description = "Route preview", body = RouteResponse),
        (status = 403, description = "Missing permission, or the module is switched off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 422, description = "Route problem rejected by the optimizer, a stop or vehicle does not fit the plan, or no station restores the plan's capacity kind (code `routing.no_station_for_capacity`)", body = ErrorBody),
        (status = 502, description = "Routing engine unavailable", body = ErrorBody),
        (status = 503, description = "Routing feature is disabled (code `feature.routing_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(module = %req.module))]
pub async fn preview_plan_route(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<PlanRouteRequest>,
) -> Result<Json<RouteResponse>, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled {
            feature: Feature::Routing,
        });
    }
    let module = ModuleKey::new(&req.module)?;
    let (access, resource) = plan_access(&state, &module)?;
    let org = scope::resolve_target_org(
        &state.user_service,
        &state.authorization_service,
        user.caller(),
        None,
    )
    .await?;
    access
        .require_write(
            user.caller(),
            Permission::new(resource, Action::Create),
            org,
        )
        .await?;
    let computed = state
        .operation_plan_service
        .preview_route(PreviewRoute {
            module,
            stops: stop_refs(&req.stops),
            transporter_id: Id::new(req.transporter_id),
            trailer_id: req.trailer_id.map(Id::new),
            transporter_capacity_used: req.transporter_capacity_used,
            start_point_name: req.start_point_name,
            organization_id: org,
        })
        .await?;
    Ok(Json(RouteResponse {
        distance: computed.route.distance.meters(),
        duration: computed.route.duration.as_secs_f64(),
        refill_count: computed.route.refill_count,
        station_visits: computed.station_visits.iter().map(Into::into).collect(),
        total_demand: computed.total_demand,
        geometry: RouteGeometry::from_coordinates(&computed.route.geometry),
    }))
}

#[utoipa::path(get, path = "/plans/{plan_id}/route/gpx", tag = "Plans",
    operation_id = "getPlanGpx",
    summary = "Download GPX file",
    description = "Renders the optimized route of a plan as a GPX track.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "GPX file", content_type = "application/gpx+xml"),
        (status = 404, description = "Plan not found or has no route", body = ErrorBody),
        (status = 503, description = "Routing feature is disabled (code `feature.routing_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_plan_gpx(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let plan = readable_plan(&state, user.caller(), id).await?;
    let geometry = plan
        .route_geometry()
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    let date = plan.date.format("%Y-%m-%d").to_string();
    let name = gpx::track_name(plan.description.as_deref(), &date);
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
