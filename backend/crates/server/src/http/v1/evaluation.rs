use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::dto::evaluation::{
            EvaluationResponse, RegionEvaluationResponse, VehicleEvaluationResponse,
        },
    },
    service::ServiceError,
};
use domain::{
    authorization::{Action, Permission, Resource},
    cluster::TreeClusterSearchQuery,
    sensor::SensorSearchQuery,
    shared::pagination::Pagination,
    tree::TreeSearchQuery,
    watering_plan::WateringPlanSearchQuery,
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
    user: AuthUserExtractor,
) -> Result<Json<EvaluationResponse>, ServiceError> {
    let count_pagination = Pagination::new(1, 1);

    let tree_visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Tree, Action::Read))
        .await?;
    let cluster_visible = state
        .authorization_service
        .visible_orgs_for(
            user.id,
            Permission::new(Resource::TreeCluster, Action::Read),
        )
        .await?;
    let sensor_visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Sensor, Action::Read))
        .await?;
    let plan_visible = state
        .authorization_service
        .visible_orgs_for(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Read),
        )
        .await?;
    let vehicle_visible = state
        .authorization_service
        .visible_orgs_for(user.id, Permission::new(Resource::Vehicle, Action::Read))
        .await?;

    let tree_page = state
        .tree_service
        .search_view(
            TreeSearchQuery {
                visible: tree_visible,
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let cluster_page = state
        .cluster_service
        .search_view(
            TreeClusterSearchQuery {
                visible: cluster_visible.clone(),
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let sensor_page = state
        .sensor_service
        .search_view(
            SensorSearchQuery {
                visible: sensor_visible,
                ..Default::default()
            },
            count_pagination,
        )
        .await?;
    let watering_plan_page = state
        .watering_plan_service
        .search_view(
            WateringPlanSearchQuery {
                visible: plan_visible.clone(),
                ..Default::default()
            },
            count_pagination,
        )
        .await?;

    let region_eval = state
        .evaluation_service
        .regions_with_watering_plan(cluster_visible.clone())
        .await?;
    let vehicle_eval = state
        .evaluation_service
        .vehicle_with_watering_plan(vehicle_visible, plan_visible.clone())
        .await?;
    let total_water = state
        .evaluation_service
        .total_consumed_water(cluster_visible)
        .await?;
    let user_count = state
        .evaluation_service
        .watering_plan_user(plan_visible)
        .await?;

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
