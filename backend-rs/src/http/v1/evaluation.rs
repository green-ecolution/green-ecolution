use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{
        cluster::TreeClusterSearchQuery, sensor::SensorSearchQuery, shared::pagination::Pagination,
        tree::TreeSearchQuery, watering_plan::WateringPlanSearchQuery,
    },
    http::{
        AppState,
        v1::dto::evaluation::{
            EvaluationResponse, RegionEvaluationResponse, VehicleEvaluationResponse,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_evaluation))
}

#[utoipa::path(get, path = "/evaluation", tag = "Evaluation",
    operation_id = "getEvaluation",
    summary = "Get evaluation data",
    description = "Returns aggregated statistics including resource counts, water consumption, and per-region/vehicle breakdowns.",
    responses(
        (status = 200, description = "Evaluation statistics", body = EvaluationResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_evaluation(
    State(state): State<Arc<AppState>>,
) -> Result<Json<EvaluationResponse>, ServiceError> {
    let count_pagination = Pagination::new(1, 1);

    let tree_page = state
        .tree_service
        .search_view(TreeSearchQuery::default(), count_pagination.clone())
        .await?;
    let cluster_page = state
        .cluster_service
        .search_view(TreeClusterSearchQuery::default(), count_pagination.clone())
        .await?;
    let sensor_page = state
        .sensor_service
        .search_view(SensorSearchQuery::default(), count_pagination.clone())
        .await?;
    let watering_plan_page = state
        .watering_plan_service
        .search_view(WateringPlanSearchQuery::default(), count_pagination)
        .await?;

    let region_eval = state
        .evaluation_service
        .regions_with_watering_plan()
        .await?;
    let vehicle_eval = state
        .evaluation_service
        .vehicle_with_watering_plan()
        .await?;
    let total_water = state.evaluation_service.total_consumed_water().await?;
    let user_count = state.evaluation_service.watering_plan_user().await?;

    let response = EvaluationResponse {
        tree_count: tree_page.total as u32,
        treecluster_count: cluster_page.total as u32,
        sensor_count: sensor_page.total as u32,
        watering_plan_count: watering_plan_page.total as i32,
        user_watering_plan_count: user_count as u32,
        total_water_consumption: total_water as u64,
        region_evaluation: region_eval
            .iter()
            .map(RegionEvaluationResponse::from)
            .collect(),
        vehicle_evaluation: vehicle_eval
            .iter()
            .map(VehicleEvaluationResponse::from)
            .collect(),
    };

    Ok(Json(response))
}
