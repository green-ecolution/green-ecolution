use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{http::AppState, service::ServiceError};

use super::dto::routing::StartPointResponse;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(list_routing_start_points))
}

#[utoipa::path(get, path = "/routing/start-points", tag = "Routing",
    operation_id = "listRoutingStartPoints",
    summary = "List routing start points",
    description = "Returns the configured named start/return points for watering routes. The first entry is the default.",
    responses(
        (status = 200, description = "List of start points", body = Vec<StartPointResponse>),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_routing_start_points(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StartPointResponse>>, ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "routing" });
    }
    Ok(Json(state.routing_start_points.clone()))
}
