use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

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

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/regions", get(list_regions))
        .route("/regions/{region_id}", get(get_region))
}

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

pub async fn get_region(
    State(state): State<Arc<AppState>>,
    Path(region_id): Path<i32>,
) -> Result<Json<RegionResponse>, ServiceError> {
    let region = state.region_service.by_id(Id::new(region_id)).await?;
    Ok(Json(RegionResponse::from(&region)))
}
