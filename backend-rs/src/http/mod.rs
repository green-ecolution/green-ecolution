use std::sync::Arc;

use axum::{Router, routing::get};

use crate::domain::region::RegionRepository;

pub mod error;
pub mod pagination;
pub mod region;

pub struct AppState {
    pub region_repo: Arc<dyn RegionRepository + Send + Sync>,
}

pub fn router(state: Arc<AppState>) -> Router {
    let v1 = Router::new()
        .route("/regions", get(region::all_region))
        .route("/regions/{region_id}", get(region::region_by_id));

    Router::new().nest("/api/v1", v1).with_state(state)
}
