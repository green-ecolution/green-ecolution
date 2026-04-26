use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    domain::info::SystemInfoProvider,
    service::{
        cluster_service::ClusterService,
        evaluation_service::EvaluationService,
        region_service::RegionService,
        sensor_service::SensorService,
        tree_service::TreeService,
        vehicle_service::VehicleService,
        watering_plan_service::WateringPlanService,
    },
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
    pub info_provider: Arc<dyn SystemInfoProvider>,
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Regions", description = "Region management"),
        (name = "Tree Clusters", description = "Tree cluster management"),
        (name = "Trees", description = "Tree management"),
        (name = "Vehicles", description = "Vehicle management"),
        (name = "Sensors", description = "Sensor management"),
        (name = "Watering Plans", description = "Watering plan management"),
        (name = "Evaluation", description = "Evaluation and statistics"),
        (name = "Info", description = "Application info"),
    )
)]
struct ApiDoc;

pub fn router(state: Arc<AppState>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", v1::router())
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
