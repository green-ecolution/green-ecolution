use std::sync::Arc;

use axum::Router;

use crate::{domain::region::RegionRepository, http};

pub mod v1;

pub struct AppState {
    pub region_repo: Arc<dyn RegionRepository>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1", http::v1::router())
        .with_state(state)
}
