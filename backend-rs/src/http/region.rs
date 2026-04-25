use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    domain::{
        Id, RepositoryError,
        region::Region,
        shared::pagination::{Page, Pagination},
    },
    http::{
        AppState,
        pagination::{PaginationParams, PaginationRepsonse},
    },
};

#[derive(serde::Serialize)]
pub struct RegionResponse {
    id: i32,
    name: String,
}

#[derive(serde::Serialize)]
pub struct RegionListResponse {
    data: Vec<RegionResponse>,
    pagination: PaginationRepsonse,
}

impl RegionListResponse {
    pub fn from_page(page: Page<Region>, current_page: u64, per_page: u64) -> Self {
        let total_pages = (page.total + per_page - 1) / per_page;
        Self {
            data: page.items.iter().map(|r| r.into()).collect(),
            pagination: PaginationRepsonse {
                total: page.total,
                current_page,
                total_pages,
                next_page: if current_page < total_pages {
                    Some(current_page + 1)
                } else {
                    None
                },
                prev_page: if current_page > 1 {
                    Some(current_page - 1)
                } else {
                    None
                },
            },
        }
    }
}

impl Into<RegionResponse> for Region {
    fn into(self) -> RegionResponse {
        RegionResponse {
            id: self.id().value(),
            name: self.name().to_string(),
        }
    }
}

impl Into<RegionResponse> for &Region {
    fn into(self) -> RegionResponse {
        RegionResponse {
            id: self.id().value(),
            name: self.name().to_string(),
        }
    }
}

pub async fn all_region(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<RegionListResponse>, RepositoryError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let region = state.region_repo.all(pagination).await?;
    let response = RegionListResponse::from_page(region, params.page, params.per_page);
    Ok(Json(response))
}

pub async fn region_by_id(
    State(state): State<Arc<AppState>>,
    Path(region_id): Path<i32>,
) -> Result<Json<RegionResponse>, RepositoryError> {
    let region = state.region_repo.by_id(Id::new(region_id)).await?;
    Ok(Json(region.into()))
}
