use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::domain::{
    cluster::TreeClusterRepository,
    evaluation::EvaluationRepository,
    region::RegionRepository,
    sensor::SensorRepository,
    tree::TreeRepository,
    vehicle::VehicleRepository,
    watering_plan::WateringPlanRepository,
};

pub mod v1;

pub struct AppState {
    pub region_repo: Arc<dyn RegionRepository>,
    pub tree_repo: Arc<dyn TreeRepository>,
    pub sensor_repo: Arc<dyn SensorRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
    pub cluster_repo: Arc<dyn TreeClusterRepository>,
    pub watering_plan_repo: Arc<dyn WateringPlanRepository>,
    pub evaluation_repo: Arc<dyn EvaluationRepository>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1", v1::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
