use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::AppState;
use crate::http::v1::dto::info::ServiceStatusResponse;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(readiness))
}

#[utoipa::path(get, path = "/health", tag = "Info",
    operation_id = "health",
    summary = "Liveness probe",
    description = "Lightweight liveness probe for container orchestrators. \
        Returns 200 OK while the HTTP server is responsive; performs no \
        downstream service checks. Use /api/ready for a readiness probe \
        that verifies critical dependencies, or /api/v1/info/services for a deep \
        services health check.",
    responses(
        (status = 200, description = "Server is alive"),
    )
)]
#[tracing::instrument(level = "trace", skip_all)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessResponse {
    #[schema(example = true)]
    pub ready: bool,
    pub services: Vec<ServiceStatusResponse>,
}

#[utoipa::path(get, path = "/ready", tag = "Info",
    operation_id = "readiness",
    summary = "Readiness probe",
    description = "Readiness probe for container orchestrators. Verifies that \
        critical dependencies (the database) are reachable and returns 200 only \
        when the service can serve traffic; returns 503 otherwise so the pod is \
        pulled from the load balancer. Optional services (auth, MQTT) are \
        reported by /api/v1/info/services and do not gate readiness.",
    responses(
        (status = 200, description = "Service is ready to serve traffic", body = ReadinessResponse),
        (status = 503, description = "A critical dependency is unavailable", body = ReadinessResponse),
    )
)]
#[tracing::instrument(level = "trace", skip_all)]
pub async fn readiness(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let readiness = state.readiness_reader.readiness().await;
    let ready = readiness.is_ready();
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = ReadinessResponse {
        ready,
        services: readiness
            .services
            .iter()
            .map(ServiceStatusResponse::from)
            .collect(),
    };
    (status, Json(body))
}
