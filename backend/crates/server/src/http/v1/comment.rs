use std::{collections::HashMap, sync::Arc};

use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path, Query},
        v1::{
            dto::{
                ListResponse,
                comment::{CommentResponse, CreateCommentRequest},
            },
            error::ErrorBody,
            pagination::PaginationParams,
            scope,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    comment::{CommentBody, CommentSubject, CommentView},
    shared::pagination::Pagination,
    user::UserView,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_cluster_comments, create_cluster_comment))
        .routes(routes!(delete_cluster_comment))
        .routes(routes!(list_plan_comments, create_plan_comment))
        .routes(routes!(delete_plan_comment))
}

/// Resolves the cluster subject after checking that the caller may see the
/// cluster at all. Returns the owning organization so the caller can require a
/// further permission in it.
async fn cluster_scope(
    state: &AppState,
    user_id: Uuid,
    cluster_id: Uuid,
) -> Result<(CommentSubject, Uuid), ServiceError> {
    let view = state
        .cluster_service
        .view_by_id(Id::new(cluster_id))
        .await?;
    let ctx = state.authorization_service.context_for(user_id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::TreeCluster, Action::Read),
        view.organization_id,
    )?;
    Ok((
        CommentSubject::TreeCluster(Id::new(cluster_id)),
        view.organization_id,
    ))
}

async fn plan_scope(
    state: &AppState,
    user_id: Uuid,
    plan_id: Uuid,
) -> Result<(CommentSubject, Uuid), ServiceError> {
    let view = state
        .watering_plan_service
        .view_by_id(Id::new(plan_id))
        .await?;
    let ctx = state.authorization_service.context_for(user_id).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(Resource::WateringPlan, Action::Read),
        view.organization_id,
    )?;
    Ok((
        CommentSubject::WateringPlan(Id::new(plan_id)),
        view.organization_id,
    ))
}

/// Checks that `comment_id` belongs to `subject` and that the caller may
/// remove it. The author may always delete their own comment; anyone else
/// needs the parent resource's delete permission. A comment addressed through
/// the wrong parent reads as 404, never 403.
async fn authorize_delete(
    state: &AppState,
    user_id: Uuid,
    subject: CommentSubject,
    org: Uuid,
    comment_id: Uuid,
    resource: Resource,
) -> Result<Id<domain::comment::Comment>, ServiceError> {
    let id = Id::new(comment_id);
    let comment = state.comment_service.by_id(id).await?;
    if comment.subject != subject {
        return Err(domain::RepositoryError::NotFound.into());
    }
    if comment.author_id != user_id {
        state
            .authorization_service
            .require(
                user_id,
                Permission::new(resource, Action::Delete),
                Id::new(org),
            )
            .await?;
    }
    Ok(id)
}

fn display_name(user: &UserView) -> String {
    let full_name = format!("{} {}", user.first_name.trim(), user.last_name.trim());
    match full_name.trim() {
        "" => user.username.as_str().to_owned(),
        name => name.to_owned(),
    }
}

/// Resolves author display names for one page in a single lookup. An IdP
/// outage must not fail the request — the comments themselves are still valid,
/// they just render without a name.
async fn resolve_author_names(state: &AppState, views: &[CommentView]) -> HashMap<Uuid, String> {
    let mut ids: Vec<Uuid> = views.iter().map(|v| v.author_id).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return HashMap::new();
    }
    state
        .user_service
        .by_ids(&ids)
        .await
        .unwrap_or_default()
        .iter()
        .map(|user| (user.id, display_name(user)))
        .collect()
}

async fn list_for(
    state: &AppState,
    subject: CommentSubject,
    params: &PaginationParams,
) -> Result<ListResponse<CommentResponse>, ServiceError> {
    let pagination = Pagination::from(params);
    let page = state.comment_service.list(subject, pagination).await?;
    let names = resolve_author_names(state, &page.items).await;
    Ok(ListResponse::from_page_with(
        page,
        &pagination,
        |view: &CommentView| CommentResponse::from_parts(view, names.get(&view.author_id).cloned()),
    ))
}

async fn create_for(
    state: &AppState,
    subject: CommentSubject,
    author_id: Uuid,
    payload: CreateCommentRequest,
) -> Result<CommentResponse, ServiceError> {
    let body = CommentBody::new(payload.body)?;
    let view = state
        .comment_service
        .create(subject, author_id, body)
        .await?;
    let names = resolve_author_names(state, std::slice::from_ref(&view)).await;
    Ok(CommentResponse::from_parts(
        &view,
        names.get(&view.author_id).cloned(),
    ))
}

#[utoipa::path(
    get,
    path = "/clusters/{cluster_id}/comments",
    tag = "Comments",
    operation_id = "listClusterComments",
    summary = "List comments on a tree cluster",
    description = "Returns the comments of a tree cluster, newest first. Requires `tree_cluster:read` in the cluster's organization.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID"), PaginationParams),
    responses(
        (status = 200, description = "Paginated list of comments", body = ListResponse<CommentResponse>),
        (status = 404, description = "Cluster not found or not visible", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id))]
pub async fn list_cluster_comments(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(cluster_id): Path<uuid::Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<CommentResponse>>, ServiceError> {
    let (subject, _org) = cluster_scope(&state, user.id, cluster_id).await?;
    Ok(Json(list_for(&state, subject, &params).await?))
}

#[utoipa::path(
    post,
    path = "/clusters/{cluster_id}/comments",
    tag = "Comments",
    operation_id = "createClusterComment",
    summary = "Comment on a tree cluster",
    description = "Adds a comment to a tree cluster. Requires `tree_cluster:update` in the cluster's organization.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
        (status = 403, description = "Missing `tree_cluster:update`", body = ErrorBody),
        (status = 404, description = "Cluster not found or not visible", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id))]
pub async fn create_cluster_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(cluster_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServiceError> {
    let (subject, org) = cluster_scope(&state, user.id, cluster_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::TreeCluster, Action::Update),
            Id::new(org),
        )
        .await?;
    let response = create_for(&state, subject, user.id, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/clusters/{cluster_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "deleteClusterComment",
    summary = "Delete a comment on a tree cluster",
    description = "Removes a comment. The author may delete their own comment; anyone else needs `tree_cluster:delete` in the cluster's organization.",
    params(
        ("cluster_id" = uuid::Uuid, Path, description = "Cluster ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 403, description = "Not the author and missing `tree_cluster:delete`", body = ErrorBody),
        (status = 404, description = "Cluster or comment not found, or the comment belongs to another cluster", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id, comment.id = %comment_id))]
pub async fn delete_cluster_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((cluster_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::http::StatusCode, ServiceError> {
    let (subject, org) = cluster_scope(&state, user.id, cluster_id).await?;
    let id = authorize_delete(
        &state,
        user.id,
        subject,
        org,
        comment_id,
        Resource::TreeCluster,
    )
    .await?;
    state.comment_service.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/watering-plans/{plan_id}/comments",
    tag = "Comments",
    operation_id = "listWateringPlanComments",
    summary = "List comments on a watering plan",
    description = "Returns the comments of a watering plan, newest first. Requires `watering_plan:read` in the plan's organization.",
    params(("plan_id" = uuid::Uuid, Path, description = "Watering plan ID"), PaginationParams),
    responses(
        (status = 200, description = "Paginated list of comments", body = ListResponse<CommentResponse>),
        (status = 404, description = "Watering plan not found or not visible", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(watering_plan.id = %plan_id))]
pub async fn list_plan_comments(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(plan_id): Path<uuid::Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<CommentResponse>>, ServiceError> {
    let (subject, _org) = plan_scope(&state, user.id, plan_id).await?;
    Ok(Json(list_for(&state, subject, &params).await?))
}

#[utoipa::path(
    post,
    path = "/watering-plans/{plan_id}/comments",
    tag = "Comments",
    operation_id = "createWateringPlanComment",
    summary = "Comment on a watering plan",
    description = "Adds a comment to a watering plan. Requires `watering_plan:update` in the plan's organization.",
    params(("plan_id" = uuid::Uuid, Path, description = "Watering plan ID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
        (status = 403, description = "Missing `watering_plan:update`", body = ErrorBody),
        (status = 404, description = "Watering plan not found or not visible", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(watering_plan.id = %plan_id))]
pub async fn create_plan_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(plan_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServiceError> {
    let (subject, org) = plan_scope(&state, user.id, plan_id).await?;
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::WateringPlan, Action::Update),
            Id::new(org),
        )
        .await?;
    let response = create_for(&state, subject, user.id, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/watering-plans/{plan_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "deleteWateringPlanComment",
    summary = "Delete a comment on a watering plan",
    description = "Removes a comment. The author may delete their own comment; anyone else needs `watering_plan:delete` in the plan's organization.",
    params(
        ("plan_id" = uuid::Uuid, Path, description = "Watering plan ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 403, description = "Not the author and missing `watering_plan:delete`", body = ErrorBody),
        (status = 404, description = "Watering plan or comment not found, or the comment belongs to another plan", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(watering_plan.id = %plan_id, comment.id = %comment_id))]
pub async fn delete_plan_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((plan_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::http::StatusCode, ServiceError> {
    let (subject, org) = plan_scope(&state, user.id, plan_id).await?;
    let id = authorize_delete(
        &state,
        user.id,
        subject,
        org,
        comment_id,
        Resource::WateringPlan,
    )
    .await?;
    state.comment_service.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
