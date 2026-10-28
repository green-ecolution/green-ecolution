use std::sync::Arc;

use axum::extract::State;
use domain::{
    Id,
    authorization::{Action, Caller, Permission},
    comment::CommentSubject,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::state::IrrigationHttpState;
use crate::{resources, targets::cluster_target};
use server::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path, Query},
        v1::{
            comment::{authorize_own_comment, create_for, list_for, update_for},
            dto::{
                ListResponse,
                comment::{CommentResponse, CreateCommentRequest, UpdateCommentRequest},
            },
            error::ErrorBody,
            pagination::PaginationParams,
            scope,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new()
        .routes(routes!(list_cluster_comments, create_cluster_comment))
        .routes(routes!(delete_cluster_comment, update_cluster_comment))
}

/// Resolves the cluster subject after checking that the caller may see the
/// cluster at all.
async fn cluster_scope(
    state: &IrrigationHttpState,
    caller: Caller,
    cluster_id: Uuid,
) -> Result<CommentSubject, ServiceError> {
    let view = state.clusters.view_by_id(Id::new(cluster_id)).await?;
    let ctx = state.authorization.context_for(caller).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resources::TREE_CLUSTER, Action::Read),
        view.organization_id,
    )?;
    Ok(CommentSubject::Target(cluster_target(Id::new(cluster_id))))
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
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(cluster_id): Path<uuid::Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<CommentResponse>>, ServiceError> {
    let subject = cluster_scope(&state, user.caller(), cluster_id).await?;
    Ok(Json(
        list_for(&state.comments, &state.users, subject, &params).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/clusters/{cluster_id}/comments",
    tag = "Comments",
    operation_id = "createClusterComment",
    summary = "Comment on a tree cluster",
    description = "Adds a comment to a tree cluster. Requires `tree_cluster:read` in the cluster's organization — anyone who may see the cluster may annotate it.",
    params(("cluster_id" = uuid::Uuid, Path, description = "Cluster ID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
        (status = 404, description = "Cluster not found or not visible", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id))]
pub async fn create_cluster_comment(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path(cluster_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServiceError> {
    let subject = cluster_scope(&state, user.caller(), cluster_id).await?;
    let response = create_for(&state.comments, &state.users, subject, user.id, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/clusters/{cluster_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "deleteClusterComment",
    summary = "Delete a comment on a tree cluster",
    description = "Removes a comment. Only the comment's author may delete it.",
    params(
        ("cluster_id" = uuid::Uuid, Path, description = "Cluster ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 403, description = "Not the author", body = ErrorBody),
        (status = 404, description = "Cluster or comment not found, or the comment belongs to another cluster", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id, comment.id = %comment_id))]
pub async fn delete_cluster_comment(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path((cluster_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::http::StatusCode, ServiceError> {
    let subject = cluster_scope(&state, user.caller(), cluster_id).await?;
    let id = authorize_own_comment(&state.comments, user.id, subject, comment_id).await?;
    state.comments.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/clusters/{cluster_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "updateClusterComment",
    summary = "Edit a comment on a tree cluster",
    description = "Replaces a comment's text. Only the comment's author may edit it.",
    params(
        ("cluster_id" = uuid::Uuid, Path, description = "Cluster ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = CommentResponse),
        (status = 403, description = "Not the author", body = ErrorBody),
        (status = 404, description = "Cluster or comment not found, or the comment belongs to another cluster", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(cluster.id = %cluster_id, comment.id = %comment_id))]
pub async fn update_cluster_comment(
    State(state): State<Arc<IrrigationHttpState>>,
    user: AuthUserExtractor,
    Path((cluster_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, ServiceError> {
    let subject = cluster_scope(&state, user.caller(), cluster_id).await?;
    let id = authorize_own_comment(&state.comments, user.id, subject, comment_id).await?;
    Ok(Json(
        update_for(&state.comments, &state.users, id, payload).await?,
    ))
}
