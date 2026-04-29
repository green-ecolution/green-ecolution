use std::sync::Arc;

use axum::{Router, http::HeaderValue};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnFailure, TraceLayer},
};
use utoipa::OpenApi;
use utoipa::openapi::Server;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    configuration::CorsSettings,
    domain::info::SystemInfoProvider,
    http::tracing::{MakeRequestUuid, REQUEST_ID_HEADER, make_span, on_response},
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

mod tracing;
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
    info(
        title = "Green Ecolution API",
        version = "0.1.0",
        description = "REST API for the Green Ecolution smart irrigation and green-space management platform. \
            Combines IoT sensor data, route optimization, and automated maintenance scheduling \
            to help municipalities manage urban greenery efficiently. \
            The system uses LoRaWAN sensors to monitor soil conditions, calculates optimal \
            watering routes, and provides fleet management capabilities.",
        contact(name = "Green Ecolution", url = "https://green-ecolution.de"),
        license(name = "AGPL-3.0", identifier = "AGPL-3.0-or-later"),
    ),
    tags(
        (name = "Regions", description = "Manage geographic regions used to group tree clusters. Regions define administrative boundaries for organizing green spaces."),
        (name = "Tree Clusters", description = "Manage tree clusters — logical groupings of trees that share soil conditions and watering schedules. Clusters are the primary unit for watering plan assignments."),
        (name = "Trees", description = "Manage individual trees including their species, location, planting year, and associated sensors. Trees can be assigned to clusters for grouped watering management."),
        (name = "Vehicles", description = "Manage watering vehicles (transporters and trailers) including their water capacity, dimensions, and availability status. Vehicles can be archived when decommissioned."),
        (name = "Sensors", description = "Access LoRaWAN sensor data for soil moisture monitoring. Sensors are linked to individual trees and provide real-time environmental readings."),
        (name = "Watering Plans", description = "Create and manage watering plans that combine tree clusters, vehicles, and optimized routes. Plans track status from planning through execution."),
        (name = "Evaluation", description = "Aggregated statistics and evaluation data across all managed resources. Provides insights on watering plan coverage by region and vehicle usage."),
        (name = "Info", description = "Application metadata including version information, server status, map configuration, service health, and data statistics."),
        (name = "Users", description = "User management and OAuth2/OIDC authentication via Keycloak. Handles login flows, token management, and user registration."),
        (name = "Plugins", description = "Plugin registration and lifecycle management. External plugins can register, authenticate, and maintain heartbeat connections."),
    )
)]
struct ApiDoc;

pub fn router(state: Arc<AppState>, base_url: &str, cors: &CorsSettings) -> Router {
    let (router, mut api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", v1::router())
        .split_for_parts();

    api.servers = Some(vec![Server::new(base_url)]);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(make_span)
        .on_response(on_response)
        .on_failure(DefaultOnFailure::new().level(::tracing::Level::ERROR));

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(cors_layer(cors))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(trace_layer)
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .with_state(state)
}

fn cors_layer(config: &CorsSettings) -> CorsLayer {
    if config.allowed_origins.iter().any(|o| o == "*") {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any)
}
