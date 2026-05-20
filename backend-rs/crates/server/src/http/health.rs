use std::sync::Arc;

use axum::http::StatusCode;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::AppState;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(health))
}

#[utoipa::path(get, path = "/health", tag = "Info",
    operation_id = "health",
    summary = "Liveness probe",
    description = "Lightweight liveness probe for container orchestrators. \
        Returns 200 OK while the HTTP server is responsive; performs no \
        downstream service checks. Use /v1/info/services for a deep \
        services health check.",
    responses(
        (status = 200, description = "Server is alive"),
    )
)]
#[tracing::instrument(level = "trace", skip_all)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}
