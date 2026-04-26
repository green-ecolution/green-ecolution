use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};

use crate::{
    domain::{
        Id,
        shared::pagination::Pagination,
        watering_plan::{WateringPlan, WateringPlanQuery},
    },
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                cluster::TreeClusterInListResponse,
                vehicle::VehicleResponse,
                watering_plan::{
                    WateringPlanCreateRequest, WateringPlanInListResponse,
                    WateringPlanInListView, WateringPlanResponse, WateringPlanUpdateRequest,
                    WateringPlanView,
                },
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/watering-plans",
            get(list_watering_plans).post(create_watering_plan),
        )
        .route(
            "/watering-plans/route/gpx/{gpx_name}",
            get(get_gpx_file),
        )
        .route("/watering-plans/route/preview", post(preview_route))
        .route(
            "/watering-plans/{watering_plan_id}",
            get(get_watering_plan)
                .put(update_watering_plan)
                .delete(delete_watering_plan),
        )
}

async fn resolve_plan_relations(
    state: &AppState,
    plan: &WateringPlan,
) -> Result<(VehicleResponse, Option<VehicleResponse>, Vec<TreeClusterInListResponse>), ServiceError>
{
    let transporter_id = plan
        .transporter_id
        .ok_or_else(|| ServiceError::InvalidInput("watering plan has no transporter".into()))?;
    let transporter = VehicleResponse::from(&state.vehicle_service.by_id(transporter_id).await?);

    let trailer = match plan.trailer_id {
        Some(id) => Some(VehicleResponse::from(
            &state.vehicle_service.by_id(id).await?,
        )),
        None => None,
    };

    let clusters = state.cluster_service.by_ids(&plan.cluster_ids).await?;
    let region_ids: Vec<_> = clusters.iter().filter_map(|c| c.region_id).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();

    let cluster_responses: Vec<TreeClusterInListResponse> = clusters
        .iter()
        .map(|c| {
            let region = c.region_id.and_then(|id| region_map.get(&id).copied());
            TreeClusterInListResponse::from((c, region))
        })
        .collect();

    Ok((transporter, trailer, cluster_responses))
}

pub async fn list_watering_plans(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<WateringPlanInListResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .watering_plan_service
        .all(WateringPlanQuery::default(), pagination)
        .await?;

    // Batch fetch all vehicles and clusters
    let vehicle_ids: Vec<_> = page
        .items
        .iter()
        .flat_map(|p| [p.transporter_id, p.trailer_id].into_iter().flatten())
        .collect();
    let vehicles = state.vehicle_service.by_ids(&vehicle_ids).await?;
    let vehicle_map: HashMap<_, _> = vehicles.iter().map(|v| (v.id, v)).collect();

    let cluster_ids: Vec<_> = page
        .items
        .iter()
        .flat_map(|p| &p.cluster_ids)
        .copied()
        .collect();
    let clusters = state.cluster_service.by_ids(&cluster_ids).await?;
    let region_ids: Vec<_> = clusters.iter().filter_map(|c| c.region_id).collect();
    let regions = state.region_service.by_ids(&region_ids).await?;
    let region_map: HashMap<_, _> = regions.iter().map(|r| (r.id, r)).collect();
    let cluster_map: HashMap<_, _> = clusters.iter().map(|c| (c.id, c)).collect();

    let response = ListResponse::from_page_with(page, params.page, params.per_page, |plan| {
        let transporter = plan
            .transporter_id
            .and_then(|id| vehicle_map.get(&id))
            .map(|v| VehicleResponse::from(*v))
            .expect("watering plan must have a transporter");
        let trailer = plan
            .trailer_id
            .and_then(|id| vehicle_map.get(&id))
            .map(|v| VehicleResponse::from(*v));
        let plan_clusters: Vec<TreeClusterInListResponse> = plan
            .cluster_ids
            .iter()
            .filter_map(|cid| cluster_map.get(cid))
            .map(|c| {
                let region = c.region_id.and_then(|rid| region_map.get(&rid).copied());
                TreeClusterInListResponse::from((*c, region))
            })
            .collect();

        WateringPlanInListResponse::from(WateringPlanInListView {
            plan: plan.clone(),
            transporter,
            trailer,
            clusters: plan_clusters,
            user_ids: vec![],
        })
    });

    Ok(Json(response))
}

pub async fn get_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let plan = state.watering_plan_service.by_id(Id::from(id)).await?;
    let (transporter, trailer, clusters) = resolve_plan_relations(&state, &plan).await?;

    let response = WateringPlanResponse::from(WateringPlanView {
        plan,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation: vec![],
    });

    Ok(Json(response))
}

pub async fn create_watering_plan(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<WateringPlanCreateRequest>,
) -> Result<(StatusCode, Json<WateringPlanResponse>), ServiceError> {
    let create = entity.try_into().map_err(ServiceError::Domain)?;
    let plan = state.watering_plan_service.create(create).await?;
    let (transporter, trailer, clusters) = resolve_plan_relations(&state, &plan).await?;

    let response = WateringPlanResponse::from(WateringPlanView {
        plan,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation: vec![],
    });

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<WateringPlanUpdateRequest>,
) -> Result<Json<WateringPlanResponse>, ServiceError> {
    let update = entity.try_into().map_err(ServiceError::Domain)?;
    let plan = state
        .watering_plan_service
        .update(Id::from(id), update)
        .await?;
    let (transporter, trailer, clusters) = resolve_plan_relations(&state, &plan).await?;

    let response = WateringPlanResponse::from(WateringPlanView {
        plan,
        transporter,
        trailer,
        clusters,
        user_ids: vec![],
        evaluation: vec![],
    });

    Ok(Json(response))
}

pub async fn delete_watering_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.watering_plan_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_gpx_file(
    State(_state): State<Arc<AppState>>,
    Path(_name): Path<String>,
) -> Result<Json<()>, ServiceError> {
    todo!()
}

pub async fn preview_route(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<()>, ServiceError> {
    todo!()
}
