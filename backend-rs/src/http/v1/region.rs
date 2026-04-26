use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{Id, region::RegionQuery, shared::pagination::Pagination},
    http::{
        AppState,
        v1::{
            dto::{ListResponse, region::RegionResponse},
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_regions))
        .routes(routes!(get_region))
}

#[utoipa::path(
    get,
    path = "/regions",
    tag = "Regions",
    operation_id = "listRegions",
    summary = "List all regions",
    description = "Returns a paginated list of all geographic regions used to organize tree clusters.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of regions", body = ListResponse<RegionResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn list_regions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<RegionResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .region_service
        .all(RegionQuery::default(), pagination)
        .await?;
    let response = ListResponse::<RegionResponse>::from_page(page, params.page, params.per_page);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/regions/{region_id}",
    tag = "Regions",
    operation_id = "getRegion",
    summary = "Get a region",
    description = "Returns a single region by its unique identifier.",
    params(("region_id" = i32, Path, description = "Region ID")),
    responses(
        (status = 200, description = "Region found", body = RegionResponse),
        (status = 404, description = "Region not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_region(
    State(state): State<Arc<AppState>>,
    Path(region_id): Path<i32>,
) -> Result<Json<RegionResponse>, ServiceError> {
    let region = state.region_service.by_id(Id::new(region_id)).await?;
    Ok(Json(RegionResponse::from(&region)))
}
