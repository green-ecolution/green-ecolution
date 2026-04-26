use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum::routing::{get, post};

use crate::{
    domain::{Id, vehicle::VehicleQuery, shared::pagination::Pagination},
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                vehicle::{
                    VehicleCreateRequest, VehicleResponse, VehicleUpdateRequest,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> utoipa_axum::router::OpenApiRouter<Arc<AppState>> {
    utoipa_axum::router::OpenApiRouter::new()
        .route("/vehicles", get(list_vehicles).post(create_vehicle))
        .route("/vehicles/archived", get(list_archived_vehicles))
        .route("/vehicles/archived/{vehicle_id}", post(archive_vehicle))
        .route("/vehicles/plate/{plate}", get(get_vehicle_by_plate))
        .route(
            "/vehicles/{vehicle_id}",
            get(get_vehicle).put(update_vehicle).delete(delete_vehicle),
        )
}

pub async fn list_vehicles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<VehicleResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .vehicle_service
        .all(VehicleQuery::default(), pagination)
        .await?;
    let response =
        ListResponse::<VehicleResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

pub async fn get_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let vehicle = state.vehicle_service.by_id(Id::from(id)).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}

pub async fn create_vehicle(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<VehicleCreateRequest>,
) -> Result<(StatusCode, Json<VehicleResponse>), ServiceError> {
    let create = entity.try_into().map_err(ServiceError::Domain)?;
    let vehicle = state.vehicle_service.create(create).await?;
    Ok((StatusCode::CREATED, Json(VehicleResponse::from(&vehicle))))
}

pub async fn update_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<VehicleUpdateRequest>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let update = entity.try_into().map_err(ServiceError::Domain)?;
    let vehicle = state.vehicle_service.update(Id::from(id), update).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}

pub async fn delete_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

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
    let response =
        ListResponse::<VehicleResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

pub async fn archive_vehicle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.vehicle_service.archive(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_vehicle_by_plate(
    State(state): State<Arc<AppState>>,
    Path(plate): Path<String>,
) -> Result<Json<VehicleResponse>, ServiceError> {
    let vehicle = state.vehicle_service.by_plate(&plate).await?;
    Ok(Json(VehicleResponse::from(&vehicle)))
}
