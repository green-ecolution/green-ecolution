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
        v1::{
            dto::{
                ListResponse,
                vehicle::{VehicleCreateRequest, VehicleResponse, VehicleUpdateRequest},
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};
use domain::{Id, shared::pagination::Pagination, vehicle::VehicleSearchQuery};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_vehicles, create_vehicle))
        .routes(routes!(list_archived_vehicles))
        .routes(routes!(archive_vehicle))
        .routes(routes!(get_vehicle_by_plate))
        .routes(routes!(get_vehicle, update_vehicle, delete_vehicle))
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
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .vehicle_service
        .search_view(VehicleSearchQuery::default(), pagination)
        .await?;
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
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let view = state.vehicle_service.view_by_id(Id::from(id)).await?;
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
    Json(entity): Json<VehicleCreateRequest>,
) -> Result<(StatusCode, Json<VehicleResponse>), ServiceError> {
    let draft = entity.into_draft()?;
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
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn update_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(entity): Json<VehicleUpdateRequest>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let update = entity.into_update()?;
    let vehicle = state.vehicle_service.replace(Id::from(id), update).await?;
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
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn delete_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.delete(Id::from(id)).await?;
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
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let query = VehicleSearchQuery {
        only_archived: true,
        with_archived: true,
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
        (status = 404, description = "Vehicle not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(vehicle.id = %id))]
pub async fn archive_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.archive(Id::from(id)).await?;
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
    Path(plate): Path<String>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let number_plate = domain::vehicle::NumberPlate::new(plate)?;
    let vehicle = state
        .vehicle_service
        .by_plate(&number_plate)
        .await?
        .ok_or(ServiceError::Repository(domain::RepositoryError::NotFound))?;
    let view = state.vehicle_service.view_by_id(vehicle.id).await?;
    Ok(Json(VehicleResponse::from(&view)))
}
