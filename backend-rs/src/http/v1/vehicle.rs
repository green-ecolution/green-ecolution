use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{Id, vehicle::VehicleQuery, shared::pagination::Pagination},
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

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_vehicles, create_vehicle))
        .routes(routes!(list_archived_vehicles))
        .routes(routes!(archive_vehicle))
        .routes(routes!(get_vehicle_by_plate))
        .routes(routes!(get_vehicle, update_vehicle, delete_vehicle))
}

#[utoipa::path(get, path = "/vehicles", tag = "Vehicles",
    params(PaginationParams),
    responses((status = 200, description = "Paginated list of vehicles"))
)]
pub async fn list_vehicles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .vehicle_service
        .all(VehicleQuery::default(), pagination)
        .await?;
    let response = ListResponse::<VehicleResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

#[utoipa::path(get, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    params(("vehicle_id" = i32, Path, description = "Vehicle ID")),
    responses(
        (status = 200, description = "Vehicle found", body = VehicleResponse),
        (status = 404, description = "Vehicle not found"),
    )
)]
pub async fn get_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let vehicle = state.vehicle_service.by_id(Id::from(id)).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}

#[utoipa::path(post, path = "/vehicles", tag = "Vehicles",
    request_body = VehicleCreateRequest,
    responses(
        (status = 201, description = "Vehicle created", body = VehicleResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Number plate already exists"),
    )
)]
pub async fn create_vehicle(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<VehicleCreateRequest>,
) -> Result<(StatusCode, Json<VehicleResponse>), ServiceError> {
    let create = entity.try_into().map_err(ServiceError::Domain)?;
    let vehicle = state.vehicle_service.create(create).await?;
    Ok((StatusCode::CREATED, Json(VehicleResponse::from(&vehicle))))
}

#[utoipa::path(put, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    params(("vehicle_id" = i32, Path, description = "Vehicle ID")),
    request_body = VehicleUpdateRequest,
    responses(
        (status = 200, description = "Vehicle updated", body = VehicleResponse),
        (status = 404, description = "Vehicle not found"),
    )
)]
pub async fn update_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<VehicleUpdateRequest>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let update = entity.try_into().map_err(ServiceError::Domain)?;
    let vehicle = state.vehicle_service.update(Id::from(id), update).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}

#[utoipa::path(delete, path = "/vehicles/{vehicle_id}", tag = "Vehicles",
    params(("vehicle_id" = i32, Path, description = "Vehicle ID")),
    responses(
        (status = 204, description = "Vehicle deleted"),
        (status = 404, description = "Vehicle not found"),
    )
)]
pub async fn delete_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/vehicles/archived", tag = "Vehicles",
    params(PaginationParams),
    responses((status = 200, description = "Paginated list of archived vehicles"))
)]
pub async fn list_archived_vehicles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let query = VehicleQuery {
        only_archived: true,
        with_archived: true,
        ..Default::default()
    };
    let page = state.vehicle_service.all(query, pagination).await?;
    let response = ListResponse::<VehicleResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

#[utoipa::path(post, path = "/vehicles/archived/{vehicle_id}", tag = "Vehicles",
    params(("vehicle_id" = i32, Path, description = "Vehicle ID")),
    responses(
        (status = 204, description = "Vehicle archived"),
        (status = 404, description = "Vehicle not found"),
    )
)]
pub async fn archive_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.archive(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/vehicles/plate/{plate}", tag = "Vehicles",
    params(("plate" = String, Path, description = "Vehicle number plate")),
    responses(
        (status = 200, description = "Vehicle found", body = VehicleResponse),
        (status = 404, description = "Vehicle not found"),
    )
)]
pub async fn get_vehicle_by_plate(
    State(state): State<Arc<AppState>>,
    Path(plate): Path<String>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let vehicle = state.vehicle_service.by_plate(&plate).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}
