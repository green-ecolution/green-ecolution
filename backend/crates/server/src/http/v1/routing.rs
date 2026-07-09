use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{http::AppState, service::ServiceError};
use domain::Id;

use super::dto::routing::{StartPointRequest, StartPointResponse};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_routing_start_points, create_start_point))
        .routes(routes!(update_start_point, delete_start_point))
        .routes(routes!(set_default_start_point))
}

fn ensure_routing(state: &AppState) -> Result<(), ServiceError> {
    if !state.feature_flags.routing_enabled {
        return Err(ServiceError::FeatureDisabled { feature: "routing" });
    }
    Ok(())
}

#[utoipa::path(get, path = "/routing/start-points", tag = "Routing",
    operation_id = "listRoutingStartPoints",
    summary = "List routing start points",
    description = "Returns the persisted named start/return points for watering routes.",
    responses(
        (status = 200, description = "List of start points", body = Vec<StartPointResponse>),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_routing_start_points(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<StartPointResponse>>, ServiceError> {
    ensure_routing(&state)?;
    let points = state.start_point_service.list().await?;
    Ok(Json(points.iter().map(StartPointResponse::from).collect()))
}

#[utoipa::path(post, path = "/routing/start-points", tag = "Routing",
    operation_id = "createStartPoint",
    summary = "Create a start point",
    request_body = StartPointRequest,
    responses(
        (status = 201, description = "Start point created", body = StartPointResponse),
        (status = 400, description = "Invalid input"),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_start_point(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartPointRequest>,
) -> Result<(StatusCode, Json<StartPointResponse>), ServiceError> {
    ensure_routing(&state)?;
    let draft = req.into_draft()?;
    let sp = state.start_point_service.create(draft).await?;
    Ok((StatusCode::CREATED, Json(StartPointResponse::from(&sp))))
}

#[utoipa::path(put, path = "/routing/start-points/{start_point_id}", tag = "Routing",
    operation_id = "updateStartPoint",
    summary = "Update a start point",
    params(("start_point_id" = uuid::Uuid, Path, description = "Start point ID")),
    request_body = StartPointRequest,
    responses(
        (status = 200, description = "Start point updated", body = StartPointResponse),
        (status = 404, description = "Start point not found"),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(start_point.id = %id))]
pub async fn update_start_point(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<StartPointRequest>,
) -> Result<Json<StartPointResponse>, ServiceError> {
    ensure_routing(&state)?;
    let update = req.into_update()?;
    let sp = state.start_point_service.update(Id::new(id), update).await?;
    Ok(Json(StartPointResponse::from(&sp)))
}

#[utoipa::path(delete, path = "/routing/start-points/{start_point_id}", tag = "Routing",
    operation_id = "deleteStartPoint",
    summary = "Delete a start point",
    params(("start_point_id" = uuid::Uuid, Path, description = "Start point ID")),
    responses(
        (status = 204, description = "Start point deleted"),
        (status = 400, description = "Cannot delete the default start point"),
        (status = 404, description = "Start point not found"),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(start_point.id = %id))]
pub async fn delete_start_point(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    ensure_routing(&state)?;
    state.start_point_service.delete(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/routing/start-points/{start_point_id}/default", tag = "Routing",
    operation_id = "setDefaultStartPoint",
    summary = "Set the default start point",
    params(("start_point_id" = uuid::Uuid, Path, description = "Start point ID")),
    responses(
        (status = 204, description = "Default set"),
        (status = 404, description = "Start point not found"),
        (status = 503, description = "Routing feature is disabled"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(start_point.id = %id))]
pub async fn set_default_start_point(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ServiceError> {
    ensure_routing(&state)?;
    state.start_point_service.set_default(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
