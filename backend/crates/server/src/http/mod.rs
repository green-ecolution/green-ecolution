use std::sync::Arc;
use std::time::Duration;

use axum::{Router, http::HeaderValue};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnFailure, TraceLayer},
};
use utoipa::OpenApi;
use utoipa::openapi::Server;
use utoipa::openapi::security::{
    AuthorizationCode, Flow, OAuth2, Scopes, SecurityRequirement, SecurityScheme,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::{SwaggerUi, oauth};

use crate::{
    configuration::{AuthSettings, CorsSettings},
    http::{
        auth::{AuthLayer, validator::TokenValidator},
        tracing::{MakeRequestUuid, REQUEST_ID_HEADER, make_span, on_response},
    },
    service::{
        authorization::AuthorizationService, cluster_service::ClusterService,
        evaluation_service::EvaluationService, organization_service::OrganizationService,
        region_service::RegionService, role_service::RoleService, sensor_service::SensorService,
        start_point_service::StartPointService, tree_service::TreeService,
        user_service::UserService, vehicle_service::VehicleService,
        watering_execution_service::WateringExecutionService,
        watering_plan_service::WateringPlanService,
    },
};
use domain::info::{HealthSnapshotReader, ReadinessReader, StatisticsReader, SystemInfoProvider};

pub mod auth;
pub mod extractors;
pub mod health;
mod tracing;
pub mod v1;

#[derive(Debug, Clone, Copy)]
pub struct FeatureFlags {
    pub routing_enabled: bool,
    pub plugins_enabled: bool,
}

/// OIDC parameters Swagger UI needs to run the PKCE login flow against
/// Keycloak. Secret-free by design — the browser uses the public client.
#[derive(Debug, Clone)]
pub struct OidcSwaggerSettings {
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Copy)]
pub struct NearestTreeLimits {
    pub max_radius_meters: f64,
    pub default_limit: u32,
    pub max_limit: u32,
}

pub struct AppState {
    pub region_service: Arc<RegionService>,
    pub tree_service: Arc<TreeService>,
    pub sensor_service: Arc<SensorService>,
    pub vehicle_service: Arc<VehicleService>,
    pub cluster_service: Arc<ClusterService>,
    pub watering_plan_service: Arc<WateringPlanService>,
    pub watering_execution_service: Arc<WateringExecutionService>,
    pub evaluation_service: Arc<EvaluationService>,
    pub user_service: Arc<UserService>,
    pub info_provider: Arc<dyn SystemInfoProvider>,
    pub health_reader: Arc<dyn HealthSnapshotReader>,
    pub readiness_reader: Arc<dyn ReadinessReader>,
    pub statistics_reader: Arc<dyn StatisticsReader>,
    pub token_validator: Arc<TokenValidator>,
    pub feature_flags: FeatureFlags,
    pub nearest_tree_limits: NearestTreeLimits,
    pub frontend_config_js: std::sync::Arc<str>,
    pub start_point_service: Arc<StartPointService>,
    pub organization_service: Arc<OrganizationService>,
    pub role_service: Arc<RoleService>,
    pub authorization_service: Arc<AuthorizationService>,
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Green Ecolution API",
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
        (name = "Users", description = "User registration and role management. Authentication is handled directly against Keycloak."),
        (name = "Plugins", description = "Plugin registration and lifecycle management. External plugins can register, authenticate, and maintain heartbeat connections."),
        (name = "Routing", description = "Routing configuration and start point management. Exposes the named depot locations available for watering route optimization."),
        (name = "Organizations", description = "Manage the organization tree used for RBAC scoping and multi-tenancy."),
        (name = "Roles", description = "Manage roles (named permission sets) and their assignment to users."),
    ),
)]
struct ApiDoc;

pub fn render_frontend_config_js(auth: &AuthSettings) -> String {
    let env = serde_json::json!({
        "VITE_AUTH_BYPASS": (!auth.enabled).to_string(),
        "VITE_OIDC_AUTHORITY": auth.issuer_url,
        "VITE_OIDC_CLIENT_ID": auth.frontend_client_id,
    });
    format!("window._env_ = {env};\n")
}

async fn frontend_config_js(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        state.frontend_config_js.to_string(),
    )
}

pub fn openapi_doc(base_url: &str) -> utoipa::openapi::OpenApi {
    let (_, mut api) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .nest("/api", health::routes())
        .nest("/api/v1", v1::public_router().merge(v1::protected_router()))
        .split_for_parts();

    rewrite_paths_for_client(&mut api, base_url);
    api
}

pub fn router(
    state: Arc<AppState>,
    base_url: &str,
    cors: &CorsSettings,
    auth_layer: AuthLayer,
    oidc: &OidcSwaggerSettings,
    request_timeout: Duration,
) -> Router {
    let (router, mut api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", health::routes())
        .nest("/api/v1", v1::router(auth_layer))
        .split_for_parts();

    rewrite_paths_for_client(&mut api, base_url);

    let router = router
        .route("/api/config.js", axum::routing::get(frontend_config_js))
        .merge(swagger_ui(api, oidc))
        // Without these, an unknown route and a wrong method answer with an
        // empty body, which a client that parses every response as JSON reads
        // as a parse failure rather than as the 404/405 it is.
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed);

    apply_middleware(router, cors, request_timeout).with_state(state)
}

/// The outer middleware stack, innermost layer first. Kept separate from
/// [`router`] so the cross-cutting behaviour can be exercised against a toy
/// router instead of the whole application.
fn apply_middleware<S>(
    router: Router<S>,
    cors: &CorsSettings,
    request_timeout: Duration,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(make_span)
        .on_response(on_response)
        .on_failure(DefaultOnFailure::new().level(::tracing::Level::ERROR));

    router
        .layer(axum::middleware::from_fn_with_state(
            request_timeout,
            enforce_timeout,
        ))
        .layer(CatchPanicLayer::custom(panic_response))
        .layer(cors_layer(cors))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(trace_layer)
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
}

/// A panic would otherwise travel up as a dropped connection: the client sees
/// a transport error instead of a 500 and the trace span records no status.
fn panic_response(panic: Box<dyn std::any::Any + Send + 'static>) -> axum::response::Response {
    let detail = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("unknown panic payload");
    ::tracing::error!(panic = detail, kind = "panic", "request handler panicked");

    v1::error::coded_error_response(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error",
        "request.panic",
    )
}

/// Bounds how long a request may occupy a connection when a downstream
/// dependency stalls. Written by hand rather than with `tower_http`'s timeout
/// layer, whose 408 carries an empty body — clients parse every response as
/// JSON.
async fn enforce_timeout(
    axum::extract::State(timeout): axum::extract::State<Duration>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    match tokio::time::timeout(timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            ::tracing::warn!(
                timeout_secs = timeout.as_secs_f64(),
                kind = "timeout",
                "request exceeded the configured timeout"
            );
            v1::error::coded_error_response(
                axum::http::StatusCode::REQUEST_TIMEOUT,
                "request timed out",
                "request.timeout",
            )
        }
    }
}

async fn route_not_found() -> axum::response::Response {
    v1::error::coded_error_response(
        axum::http::StatusCode::NOT_FOUND,
        "no such endpoint",
        "request.unknown_endpoint",
    )
}

async fn method_not_allowed() -> axum::response::Response {
    v1::error::coded_error_response(
        axum::http::StatusCode::METHOD_NOT_ALLOWED,
        "method not allowed for this endpoint",
        "request.method_not_allowed",
    )
}

const OIDC_SECURITY_SCHEME: &str = "keycloak";

/// Builds the Swagger UI, adding an OAuth2 authorization-code security scheme
/// plus a global requirement so every endpoint offers login and sends the
/// bearer token. Skipped entirely when auth is disabled (demo bypass).
///
/// We pin the explicit authorization-code flow (not an OpenID Connect discovery
/// scheme) so Swagger only ever offers PKCE login. A discovery scheme would let
/// Swagger present Keycloak's `password` grant, which the public frontend client
/// rejects (Direct Access Grants disabled → 400 unauthorized_client).
fn swagger_ui(mut api: utoipa::openapi::OpenApi, oidc: &OidcSwaggerSettings) -> SwaggerUi {
    if !oidc.enabled {
        return SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api);
    }

    let issuer = oidc.issuer_url.trim_end_matches('/');
    let scheme = SecurityScheme::OAuth2(OAuth2::new([Flow::AuthorizationCode(
        AuthorizationCode::new(
            format!("{issuer}/protocol/openid-connect/auth"),
            format!("{issuer}/protocol/openid-connect/token"),
            Scopes::from_iter([
                ("openid", "OpenID Connect"),
                ("profile", "User profile"),
                ("email", "User email"),
            ]),
        ),
    )]));
    let components = api.components.get_or_insert_with(Default::default);
    components.add_security_scheme(OIDC_SECURITY_SCHEME, scheme);
    api.security = Some(vec![SecurityRequirement::new(
        OIDC_SECURITY_SCHEME,
        Vec::<String>::new(),
    )]);

    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api)
        .oauth(
            oauth::Config::new()
                .client_id(&oidc.client_id)
                .use_pkce_with_authorization_code_grant(true)
                .scopes(vec![
                    "openid".to_owned(),
                    "profile".to_owned(),
                    "email".to_owned(),
                ]),
        )
}

fn rewrite_paths_for_client(api: &mut utoipa::openapi::OpenApi, base_url: &str) {
    let rewritten: utoipa::openapi::path::PathsMap<_, _> = std::mem::take(&mut api.paths.paths)
        .into_iter()
        .map(|(key, item)| {
            let new_key = key.strip_prefix("/api").map(String::from).unwrap_or(key);
            (new_key, item)
        })
        .collect();
    api.paths.paths = rewritten;

    let server_url = format!("{}/api", base_url.trim_end_matches('/'));
    api.servers = Some(vec![Server::new(server_url)]);
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

#[cfg(test)]
mod middleware_tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    fn permissive_cors() -> CorsSettings {
        CorsSettings {
            allowed_origins: vec!["*".to_string()],
        }
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("every response must parse as JSON")
    }

    /// Without a catch-panic layer axum drops the connection, which a client
    /// sees as a transport error rather than as the 500 it is — and the trace
    /// span never records a status.
    async fn boom() -> &'static str {
        panic!("handler exploded")
    }

    #[tokio::test]
    async fn a_panicking_handler_answers_with_a_json_500() {
        let app = apply_middleware(
            Router::new().route("/boom", get(boom)),
            &permissive_cors(),
            Duration::from_secs(30),
        );

        let response = app
            .oneshot(Request::builder().uri("/boom").body(Body::empty()).unwrap())
            .await
            .expect("the panic must not tear down the connection");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["code"], "request.panic");
    }

    #[tokio::test]
    async fn a_request_outliving_the_timeout_answers_with_json() {
        let app = apply_middleware(
            Router::new().route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    "never"
                }),
            ),
            &permissive_cors(),
            Duration::from_millis(50),
        );

        let response = app
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        let body = body_json(response).await;
        assert_eq!(body["code"], "request.timeout");
    }

    #[tokio::test]
    async fn a_handler_within_the_timeout_is_untouched() {
        let app = apply_middleware(
            Router::new().route("/fast", get(|| async { "ok" })),
            &permissive_cors(),
            Duration::from_secs(30),
        );

        let response = app
            .oneshot(Request::builder().uri("/fast").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
