use std::sync::Arc;

use server::http::v1::error::ErrorBody;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{
    dto::evaluation::{EvaluationResponse, RegionEvaluationResponse, VehicleEvaluationResponse},
    state::IrrigationHttpState,
};
use domain::{
    authorization::{Action, Permission, ResourceKey},
    operation_plan::OperationPlanSearchQuery,
    sensor::SensorSearchQuery,
    shared::pagination::Pagination,
};
use irrigation_domain::{cluster::TreeClusterSearchQuery, tree::TreeSearchQuery};
use server::{http::auth::extractor::AuthUserExtractor, service::ServiceError};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new().routes(routes!(get_evaluation))
}

#[utoipa::path(get, path = "/evaluation", tag = "Evaluation",
    operation_id = "getEvaluation",
    summary = "Get evaluation data",
    description = "Returns aggregated statistics including resource counts, water consumption, and per-region/vehicle breakdowns.",
    responses(
        (status = 200, description = "Evaluation statistics", body = EvaluationResponse),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_evaluation(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
) -> Result<Json<EvaluationResponse>, ServiceError> {
    let count_pagination = Pagination::new(1, 1);

    let tree_visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE, Action::Read),
        )
        .await?;
    let cluster_visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::TREE_CLUSTER, Action::Read),
        )
        .await?;
    let sensor_visible = state
        .authorization
        .visible_orgs_for(user.id, Permission::new(ResourceKey::SENSOR, Action::Read))
        .await?;
    let plan_visible = state
        .authorization
        .visible_orgs_for(
            user.id,
            Permission::new(crate::resources::WATERING_PLAN, Action::Read),
        )
        .await?;
    let vehicle_visible = state
        .authorization
        .visible_orgs_for(user.id, Permission::new(ResourceKey::VEHICLE, Action::Read))
        .await?;

    let tree_page = state
        .trees
        .search_view(
            TreeSearchQuery {
                visible: tree_visible,
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let cluster_page = state
        .clusters
        .search_view(
            TreeClusterSearchQuery {
                visible: cluster_visible.clone(),
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let sensor_page = state
        .sensors
        .search_view(
            SensorSearchQuery {
                visible: sensor_visible,
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let watering_plan_page = state
        .plans
        .search_view(
            OperationPlanSearchQuery {
                module: crate::key(),
                provider: None,
                statuses: Vec::new(),
                visible: plan_visible.clone(),
            },
            count_pagination,
        )
        .await?;

    let region_eval = state
        .evaluation
        .regions_with_watering_plan(cluster_visible.clone())
        .await?;
    let vehicle_eval = state
        .evaluation
        .vehicle_with_watering_plan(vehicle_visible, plan_visible.clone())
        .await?;
    let total_water = state
        .evaluation
        .total_consumed_water(cluster_visible)
        .await?;
    let user_count = state.evaluation.watering_plan_user(plan_visible).await?;

    let response = EvaluationResponse {
        tree_count: tree_page.page.total as u32,
        treecluster_count: cluster_page.page.total as u32,
        sensor_count: sensor_page.page.total as u32,
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
