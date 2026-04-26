use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::service::{
    cluster_service::ClusterService,
    evaluation_service::EvaluationService,
    region_service::RegionService,
    sensor_service::SensorService,
    tree_service::TreeService,
    vehicle_service::VehicleService,
    watering_plan_service::WateringPlanService,
};

pub mod v1;

pub struct AppState {
    pub region_service: Arc<RegionService>,
    pub tree_service: Arc<TreeService>,
    pub sensor_service: Arc<SensorService>,
    pub vehicle_service: Arc<VehicleService>,
    pub cluster_service: Arc<ClusterService>,
    pub watering_plan_service: Arc<WateringPlanService>,
    pub evaluation_service: Arc<EvaluationService>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1", v1::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
