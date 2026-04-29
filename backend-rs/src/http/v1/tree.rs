use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    domain::{
        Id,
        shared::pagination::Pagination,
        tree::{Tree, TreeQuery},
    },
    http::{
        AppState,
        v1::{
            dto::{
                ListResponse,
                tree::{
                    NearestTreeListResponse, NearestTreeParams, TreeCreateRequest, TreeResponse,
                    TreeUpdateRequest,
                },
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
        .routes(routes!(get_nearest_trees))
        .routes(routes!(get_tree, update_tree, delete_tree))
}

async fn build_tree_response(state: &AppState, tree: &Tree) -> Result<TreeResponse, ServiceError> {
    let sensor = match &tree.sensor_id {
        Some(sid) => Some(state.sensor_service.by_id(sid).await?),
        None => None,
    };
    Ok(TreeResponse::from((tree, sensor.as_ref())))
}

#[utoipa::path(get, path = "/trees", tag = "Trees",
    operation_id = "listTrees",
    summary = "List all trees",
    description = "Returns a paginated list of all trees with their associated sensor data.",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of trees", body = ListResponse<TreeResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_trees(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<TreeResponse>>, ServiceError> {
    let pagination = Pagination::from(&params);
    let page = state
        .tree_service
        .all(TreeQuery::default(), pagination)
        .await?;

    let sensor_ids: Vec<String> = page
        .items
        .iter()
        .filter_map(|t| t.sensor_id.clone())
        .collect();
    let sensors = state.sensor_service.by_ids(&sensor_ids).await?;
    let sensor_map: HashMap<&str, _> = sensors.iter().map(|s| (s.id.as_str(), s)).collect();

    let response = ListResponse::from_page_with(page, &pagination, |tree: &Tree| {
        let sensor = tree
            .sensor_id
            .as_deref()
            .and_then(|id| sensor_map.get(id).copied());
        TreeResponse::from((tree, sensor))
    });
    Ok(Json(response))
}

#[utoipa::path(get, path = "/trees/{tree_id}", tag = "Trees",
    operation_id = "getTree",
    summary = "Get a tree by ID",
    description = "Returns a single tree by its ID, including associated sensor data.",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 200, description = "Tree found", body = TreeResponse),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
pub async fn get_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<TreeResponse>, ServiceError> {
    let tree = state.tree_service.by_id(Id::from(id)).await?;
    let response = build_tree_response(&state, &tree).await?;
    Ok(Json(response))
}

#[utoipa::path(post, path = "/trees", tag = "Trees",
    operation_id = "createTree",
    summary = "Create a new tree",
    description = "Creates a new tree with location and optional cluster or sensor association.",
    request_body = TreeCreateRequest,
    responses(
        (status = 201, description = "Tree created", body = TreeResponse),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
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
    operation_id = "updateTree",
    summary = "Update a tree",
    description = "Performs a full replacement update of a tree by its ID.",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    request_body = TreeUpdateRequest,
    responses(
        (status = 200, description = "Tree updated", body = TreeResponse),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
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
    operation_id = "deleteTree",
    summary = "Delete a tree",
    description = "Permanently deletes a tree by its ID.",
    params(("tree_id" = i32, Path, description = "Tree ID")),
    responses(
        (status = 204, description = "Tree deleted"),
        (status = 404, description = "Tree not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(tree.id = id))]
pub async fn delete_tree(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ServiceError> {
    state.tree_service.delete(Id::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/trees/planting-years", tag = "Trees",
    operation_id = "listPlantingYears",
    summary = "List distinct planting years",
    description = "Returns a list of distinct planting years across all trees.",
    responses(
        (status = 200, description = "List of distinct planting years", body = Vec<i32>),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_planting_years(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<i32>>, ServiceError> {
    let years = state.tree_service.distinct_planting_years().await?;
    Ok(Json(years.into_iter().map(|y| y.year() as i32).collect()))
}

#[utoipa::path(get, path = "/trees/nearest", tag = "Trees",
    operation_id = "getNearestTrees",
    summary = "Get nearest trees",
    description = "Finds trees closest to a given coordinate.",
    params(NearestTreeParams),
    responses(
        (status = 200, description = "Nearest trees", body = NearestTreeListResponse),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_nearest_trees(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<NearestTreeParams>,
) -> Result<Json<NearestTreeListResponse>, ServiceError> {
    todo!()
}
