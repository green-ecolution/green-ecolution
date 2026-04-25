use std::sync::Arc;

use axum::{Router, routing::get};

use crate::domain::region::RegionRepository;

pub mod error;
pub mod region;

#[derive(serde::Serialize)]
pub struct PaginationRepsonse {
    total: u64,
    current_page: u32,
    total_pages: u32,
    next_page: Option<u32>,
    prev_page: Option<u32>,
}

pub struct AppState {
    pub region_repo: Arc<dyn RegionRepository + Send + Sync>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/regions",
            get(region::list_region).post(region::create_region),
        )
        .with_state(state)
}
