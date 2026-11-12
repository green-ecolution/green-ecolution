use std::{sync::Arc, time::Instant};

use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use domain::shared::coordinates::Coordinate;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{error::ErrorBody, plan::readable_plan},
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(navigate_plan))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NavigationRequest {
    pub lat: f64,
    pub lon: f64,
    /// Degrees clockwise from north.
    #[schema(nullable, minimum = 0.0, maximum = 360.0)]
    pub heading: Option<f64>,
    /// Itinerary indices still to drive, in the order to drive them. Routing covers
    /// at most the next 10 entries that have a position.
    pub remaining: Vec<usize>,
}

#[utoipa::path(post, path = "/plans/{plan_id}/navigation", tag = "Plans",
    operation_id = "navigatePlan",
    summary = "Turn-by-turn route over the remaining entries",
    description = "Routes from the given position over the listed itinerary entries and routes over at most the next 10 positioned entries (request again as the tour progresses) and returns the routing engine's answer unchanged (Valhalla, OSRM format with banner and voice instructions in German).",
    params(("plan_id" = Uuid, Path, description = "Plan ID")),
    request_body = NavigationRequest,
    responses(
        (status = 200, description = "Valhalla route (OSRM format)", body = Object),
        (status = 400, description = "Invalid indices, position or plan state (code `request.invalid_input` / `request.validation_failed`)", body = ErrorBody),
        (status = 404, description = "Plan not found", body = ErrorBody),
        (status = 422, description = "No route between the positions (code `routing.invalid_problem`)", body = ErrorBody),
        (status = 429, description = "Called again too soon (code `routing.too_many_requests`)", body = ErrorBody),
        (status = 502, description = "Routing engine unavailable (code `routing.unavailable`)", body = ErrorBody),
        (status = 503, description = "Routing feature is disabled (code `feature.routing_disabled`)", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn navigate_plan(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
    Json(body): Json<NavigationRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let plan = readable_plan(&state, user.caller(), id).await?;
    if !state
        .navigation_throttle
        .admit(user.caller().user_id, Instant::now())
    {
        return Err(ServiceError::TooManyRequests);
    }
    let origin = Coordinate::new(body.lat, body.lon)?;
    if body.heading.is_some_and(|h| !(0.0..=360.0).contains(&h)) {
        return Err(ServiceError::InvalidInput(
            "heading must be within 0..=360".into(),
        ));
    }
    let stops = state
        .operation_plan_service
        .resolved_stops_lenient(&plan)
        .await?;
    let route = state
        .navigation_service
        .route(&plan, &stops, &body.remaining, origin, body.heading)
        .await?;
    Ok(([(header::CONTENT_TYPE, "application/json")], route.json))
}
