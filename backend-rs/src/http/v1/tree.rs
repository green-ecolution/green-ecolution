use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{Id, tree::{Tree, TreeQuery}, shared::pagination::Pagination},
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                tree::{TreeCreateRequest, TreeResponse, TreeUpdateRequest},
            },
            pagination::PaginationParams,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_trees, create_tree))
        .routes(routes!(list_planting_years))
        .routes(routes!(get_tree, update_tree, delete_tree))
        .routes(routes!(get_tree_sensor))
}

async fn build_tree_response(
    state: &AppState,
    tree: &Tree,
) -> Result<TreeResponse, ServiceError> {
    let sensor = match &tree.sensor_id {
        Some(sid) => Some(state.sensor_service.by_id(sid).await?),
        None => None,
    };
    Ok(TreeResponse::from((tree, sensor.as_ref())))
}

#[utoipa::path(get, path = "/trees", tag = "Trees",
    params(PaginationParams),
    responses((status = 200, description = "Paginated list of trees"))
)]
pub async fn list_trees(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let page = state
        .tree_service
        .all(TreeQuery::default(), pagination)
        .await?;

    let sensor_ids: Vec<String> = page.items.iter().filter_map(|t| t.sensor_id.clone()).collect();
    let sensors = state.sensor_service.by_ids(&sensor_ids).await?;
    let sensor_map: HashMap<&str, _> = sensors.iter().map(|s| (s.id.as_str(), s)).collect();

    let response = ListResponse::from_page_with(page, params.page, params.per_page, |tree| {
        let sensor = tree
            .sensor_id
            .as_deref()
            .and_then(|id| sensor_map.get(id).copied());
        TreeResponse::from((tree, sensor))
    });
    Ok(Json(response))
}

#[utoipa::path(get, path = "/trees/{tree_id}", tag = "Trees",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Tree not found"),
    )
)]
pub async fn get_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let tree = state.tree_service.by_id(Id::from(id)).await?;
    let response = build_tree_response(&state, &tree).await?;
    Ok(Json(response))
}

#[utoipa::path(post, path = "/trees", tag = "Trees",
    request_body = TreeCreateRequest,
    responses(
        (status = 201, description = "Tree created", body = TreeResponse),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_tree(
    State(state): State<Arc<AppState>>,
    Json(entity): Json<TreeCreateRequest>,
) -> Result<(StatusCode, Json<TreeResponse>), ServiceError> {
    let create = entity.try_into().map_err(ServiceError::Domain)?;
    let tree = state.tree_service.create(create).await?;
    let response = build_tree_response(&state, &tree).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(put, path = "/trees/{tree_id}", tag = "Trees",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    request_body = TreeUpdateRequest,
    responses(
        (status = 200, description = "Tree updated", body = TreeResponse),
        (status = 404, description = "Tree not found"),
    )
)]
pub async fn update_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(entity): Json<TreeUpdateRequest>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let update = entity.try_into().map_err(ServiceError::Domain)?;
    let tree = state.tree_service.update(Id::from(id), update).await?;
    let response = build_tree_response(&state, &tree).await?;
    Ok(Json(response))
}

#[utoipa::path(delete, path = "/trees/{tree_id}", tag = "Trees",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 204, description = "Tree deleted"),
        (status = 404, description = "Tree not found"),
    )
)]
pub async fn delete_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.tree_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/trees/planting-years", tag = "Trees",
    responses((status = 200, description = "List of distinct planting years"))
)]
pub async fn list_planting_years(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<i32>>, ServiceError> {
    let years = state.tree_service.distinct_planting_years().await?;
    Ok(Json(years.into_iter().map(|y| y.year() as i32).collect()))
}

#[utoipa::path(get, path = "/trees/{tree_id}/sensors/{sensor_id}", tag = "Trees",
    params(
        ("tree_id" = i32, Path, description = "Tree ID"),
        ("sensor_id" = String, Path, description = "Sensor ID"),
    ),
    responses(
        (status = 200, description = "Tree with sensor", body = TreeResponse),
        (status = 404, description = "Tree not found"),
    )
)]
pub async fn get_tree_sensor(
    State(state): State<Arc<AppState>>,
    Path((tree_id, _sensor_id)): Path<(i32, String)>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let tree = state.tree_service.by_id(Id::from(tree_id)).await?;
    let response = build_tree_response(&state, &tree).await?;
    Ok(Json(response))
}
