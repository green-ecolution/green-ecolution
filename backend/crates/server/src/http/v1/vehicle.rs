use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::{
            dto::{
                ListResponse,
                tree::TransferRequest,
                vehicle::{VehicleCreateRequest, VehicleResponse, VehicleUpdateRequest},
            },
            pagination::PaginationParams,
            scope,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    shared::pagination::Pagination,
    vehicle::VehicleSearchQuery,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_vehicles, create_vehicle))
        .routes(routes!(list_archived_vehicles))
        .routes(routes!(archive_vehicle))
        .routes(routes!(get_vehicle_by_plate))
        .routes(routes!(get_vehicle, update_vehicle, delete_vehicle))
        .routes(routes!(transfer_vehicle))
}

#[utoipa::path(get, path = "/vehicles", tag = "Vehicles",
    operation_id = "listVehicles",
    summary = "List all vehicles",
    description = "Returns a paginated list of active vehicles.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of vehicles", body = ListResponse<VehicleResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_vehicles(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Vehicle, Action::Read))
        .await?;
    let query = VehicleSearchQuery {
        visible,
        ..VehicleSearchQuery::default()
    };
    let page = state.vehicle_service.search_view(query, pagination).await?;
    let response = ListResponse::<VehicleResponse>::from_page(page, &pagination);
    Ok(Json(response))
}

#[utoipa::path(get, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    operation_id = "getVehicle",
    summary = "Get a vehicle",
    description = "Returns a single vehicle by its ID.",
    params(("vehicle_id" = uuid::Uuid, Path, description = "Vehicle ID")),
    responses(
        (status = 200, description = "Vehicle found", body = VehicleResponse),
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn get_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let view = state.vehicle_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &scope::effective_orgs(view.organization_id, &[]),
    )?;
    Ok(Json(VehicleResponse::from(&view)))
}

#[utoipa::path(post, path = "/vehicles", tag = "Vehicles",
    operation_id = "createVehicle",
    summary = "Create a vehicle",
    description = "Registers a new watering vehicle.",
    request_body = VehicleCreateRequest,
    responses(
        (status = 201, description = "Vehicle created", body = VehicleResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Number plate already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(entity): Json<VehicleCreateRequest>,
) -> Result<(StatusCode, Json<VehicleResponse>), ServiceError> {
    let org = scope::resolve_target_org(&state, user.id, entity.organization_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Vehicle, Action::Create),
            org,
        )
        .await?;
    let draft = entity.into_draft(org)?;
    let vehicle = state.vehicle_service.create(draft).await?;
    let view = state.vehicle_service.view_by_id(vehicle.id).await?;
    Ok((StatusCode::CREATED, Json(VehicleResponse::from(&view))))
}

#[utoipa::path(put, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    operation_id = "updateVehicle",
    summary = "Update a vehicle",
    description = "Updates the details of an existing vehicle.",
    params(("vehicle_id" = uuid::Uuid, Path, description = "Vehicle ID")),
    request_body = VehicleUpdateRequest,
    responses(
        (status = 200, description = "Vehicle updated", body = VehicleResponse),
        (status = 403, description = "Missing vehicle:update in the owning organization"),
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn update_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<VehicleUpdateRequest>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let current = state.vehicle_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let effective_orgs = scope::effective_orgs(current.organization_id, &[]);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &effective_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Vehicle, Action::Update),
            &effective_orgs,
        )
        .await?;
    let update = entity.into_update()?;
    let vehicle = state.vehicle_service.replace(Id::new(id), update).await?;
    let view = state.vehicle_service.view_by_id(vehicle.id).await?;
    Ok(Json(VehicleResponse::from(&view)))
}

#[utoipa::path(delete, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    operation_id = "deleteVehicle",
    summary = "Delete a vehicle",
    description = "Permanently deletes a vehicle.",
    params(("vehicle_id" = uuid::Uuid, Path, description = "Vehicle ID")),
    responses(
        (status = 204, description = "Vehicle deleted"),
        (status = 403, description = "Missing vehicle:delete in the owning organization"),
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn delete_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let current = state.vehicle_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &scope::effective_orgs(current.organization_id, &[]),
    )?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Vehicle, Action::Delete),
            Id::new(current.organization_id),
        )
        .await?;
    state.vehicle_service.delete(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/vehicles/archived", tag = "Vehicles",
    operation_id = "listArchivedVehicles",
    summary = "List archived vehicles",
    description = "Returns a paginated list of decommissioned vehicles.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of archived vehicles", body = ListResponse<VehicleResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_archived_vehicles(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Vehicle, Action::Read))
        .await?;
    let query = VehicleSearchQuery {
        only_archived: true,
        with_archived: true,
        visible,
        ..Default::default()
    };
    let page = state.vehicle_service.search_view(query, pagination).await?;
    let response = ListResponse::<VehicleResponse>::from_page(page, &pagination);
    Ok(Json(response))
}

#[utoipa::path(post, path = "/vehicles/archived/{vehicle_id}", tag = "Vehicles",
    operation_id = "archiveVehicle",
    summary = "Archive a vehicle",
    description = "Moves a vehicle to the archived state.",
    params(("vehicle_id" = uuid::Uuid, Path, description = "Vehicle ID")),
    responses(
        (status = 204, description = "Vehicle archived"),
        (status = 403, description = "Missing vehicle:update in the owning organization"),
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn archive_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    let current = state.vehicle_service.view_by_id(Id::new(id)).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    let effective_orgs = scope::effective_orgs(current.organization_id, &[]);
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &effective_orgs,
    )?;
    state
        .authorization_service
        .require_any_of(
            user.id,
            Permission::new(Resource::Vehicle, Action::Update),
            &effective_orgs,
        )
        .await?;
    state.vehicle_service.archive(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/vehicles/plate/{plate}", tag = "Vehicles",
    operation_id = "getVehicleByPlate",
    summary = "Get vehicle by plate",
    description = "Looks up a vehicle by its license plate number.",
    params(("plate" = String, Path, description = "Vehicle number plate")),
    responses(
        (status = 200, description = "Vehicle found", body = VehicleResponse),
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.plate = %plate))]
pub async fn get_vehicle_by_plate(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(plate): Path<String>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let number_plate = domain::vehicle::NumberPlate::new(plate)?;
    let vehicle = state
        .vehicle_service
        .by_plate(&number_plate)
        .await?
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    let view = state.vehicle_service.view_by_id(vehicle.id).await?;
    let ctx = state.authorization_service.context_for(user.id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::Vehicle, Action::Read),
        &scope::effective_orgs(view.organization_id, &[]),
    )?;
    Ok(Json(VehicleResponse::from(&view)))
}

#[utoipa::path(patch, path = "/vehicles/{vehicle_id}/organization", tag = "Vehicles",
    operation_id = "transferVehicle", summary = "Transfer a vehicle's ownership to another organization",
    description = "Moves a vehicle to a different owning organization. Requires `vehicle:update` \
                   in both the source and target organization.",
    params(("vehicle_id" = uuid::Uuid, Path, description = "Vehicle ID")),
    request_body = TransferRequest,
    responses(
        (status = 204, description = "Vehicle transferred"),
        (status = 403, description = "Missing vehicle:update in source or target organization"),
        (status = 404, description = "Vehicle or organization not found"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn transfer_vehicle(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = state.vehicle_service.view_by_id(Id::new(id)).await?;
    let perm = Permission::new(Resource::Vehicle, Action::Update);
    state
        .authorization_service
        .require(user.id, perm, Id::new(current.organization_id))
        .await?;
    state
        .authorization_service
        .require(user.id, perm, Id::new(req.organization_id))
        .await?;
    state
        .vehicle_service
        .transfer(Id::new(id), Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
