use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    domain::{
        RepositoryError,
        region::{Region, RegionCreate},
        shared::pagination::Page,
    },
    http::{AppState, PaginationRepsonse},
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

impl Into<RegionListResponse> for Page<Region> {
    fn into(self) -> RegionListResponse {
        let data = self.items.iter().map(|r| r.into()).collect();
        let pagination = PaginationRepsonse {
            total: self.total,
            current_page: 0,
            total_pages: 0,
            next_page: None,
            prev_page: None,
        };
        RegionListResponse { data, pagination }
    }
}

pub async fn list_region(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RegionListResponse>, RepositoryError> {
    let region = state.region_repo.all().await?;
    Ok(Json(region.into()))
}

pub async fn create_region(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegionCreate>,
) -> Result<Json<RegionResponse>, RepositoryError> {
    let region = state.region_repo.create(body).await?;
    Ok(Json(region.into()))
}
