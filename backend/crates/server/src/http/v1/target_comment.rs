use std::sync::Arc;

use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
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
use domain::{
    authorization::{Action, Caller, Permission},
    comment::CommentSubject,
    target::TargetRef,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_target_comments, create_target_comment))
        .routes(routes!(delete_target_comment, update_target_comment))
}

/// Resolves the target subject after checking that it takes comments and that
/// the caller may see it at all.
async fn target_scope(
    state: &AppState,
    caller: Caller,
    module: &str,
    kind: &str,
    target_id: Uuid,
) -> Result<CommentSubject, ServiceError> {
    let target = TargetRef::new(module, kind, target_id)?;
    let admitted = state.comment_service.admit_target(&target).await?;
    let ctx = state.authorization_service.context_for(caller).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(admitted.resource, Action::Read),
        admitted.organization_id.value(),
    )?;
    Ok(CommentSubject::Target(target))
}

#[utoipa::path(
    get,
    path = "/targets/{module}/{kind}/{target_id}/comments",
    tag = "Comments",
    operation_id = "listTargetComments",
    summary = "List comments on a target",
    description = "Returns the comments of a commentable target of any module, newest first. Requires read permission on the target's resource in the target's organization.",
    params(
        ("module" = String, Path, description = "Module key"),
        ("kind" = String, Path, description = "Target kind"),
        ("target_id" = uuid::Uuid, Path, description = "Target ID"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of comments", body = ListResponse<CommentResponse>),
        (status = 403, description = "The target's module is disabled (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Target not found or not visible", body = ErrorBody),
        (status = 422, description = "The target kind takes no comments (code `target.not_accepted`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(target.id = %target_id))]
pub async fn list_target_comments(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((module, kind, target_id)): Path<(String, String, uuid::Uuid)>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<CommentResponse>>, ServiceError> {
    let subject = target_scope(&state, user.caller(), &module, &kind, target_id).await?;
    Ok(Json(
        list_for(
            &state.comment_service,
            &state.user_service,
            subject,
            &params,
        )
        .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/targets/{module}/{kind}/{target_id}/comments",
    tag = "Comments",
    operation_id = "createTargetComment",
    summary = "Comment on a target",
    description = "Adds a comment to a commentable target of any module. Requires read permission on the target's resource in the target's organization: anyone who may see the target may annotate it.",
    params(
        ("module" = String, Path, description = "Module key"),
        ("kind" = String, Path, description = "Target kind"),
        ("target_id" = uuid::Uuid, Path, description = "Target ID"),
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 403, description = "The target's module is disabled (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Target not found or not visible", body = ErrorBody),
        (status = 422, description = "The target kind takes no comments (code `target.not_accepted`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(target.id = %target_id))]
pub async fn create_target_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((module, kind, target_id)): Path<(String, String, uuid::Uuid)>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServiceError> {
    let subject = target_scope(&state, user.caller(), &module, &kind, target_id).await?;
    let response = create_for(
        &state.comment_service,
        &state.user_service,
        subject,
        user.id,
        payload,
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/targets/{module}/{kind}/{target_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "deleteTargetComment",
    summary = "Delete a comment on a target",
    description = "Removes a comment. Only the comment's author may delete it.",
    params(
        ("module" = String, Path, description = "Module key"),
        ("kind" = String, Path, description = "Target kind"),
        ("target_id" = uuid::Uuid, Path, description = "Target ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 403, description = "Not the author, or the target's module is disabled", body = ErrorBody),
        (status = 404, description = "Target or comment not found, or the comment belongs to another target", body = ErrorBody),
        (status = 422, description = "The target kind takes no comments (code `target.not_accepted`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(target.id = %target_id, comment.id = %comment_id))]
pub async fn delete_target_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((module, kind, target_id, comment_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::http::StatusCode, ServiceError> {
    let subject = target_scope(&state, user.caller(), &module, &kind, target_id).await?;
    let id = authorize_own_comment(&state.comment_service, user.id, subject, comment_id).await?;
    state.comment_service.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/targets/{module}/{kind}/{target_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "updateTargetComment",
    summary = "Edit a comment on a target",
    description = "Replaces a comment's text. Only the comment's author may edit it.",
    params(
        ("module" = String, Path, description = "Module key"),
        ("kind" = String, Path, description = "Target kind"),
        ("target_id" = uuid::Uuid, Path, description = "Target ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = CommentResponse),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 403, description = "Not the author, or the target's module is disabled", body = ErrorBody),
        (status = 404, description = "Target or comment not found, or the comment belongs to another target", body = ErrorBody),
        (status = 422, description = "The target kind takes no comments (code `target.not_accepted`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(target.id = %target_id, comment.id = %comment_id))]
pub async fn update_target_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((module, kind, target_id, comment_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, ServiceError> {
    let subject = target_scope(&state, user.caller(), &module, &kind, target_id).await?;
    let id = authorize_own_comment(&state.comment_service, user.id, subject, comment_id).await?;
    Ok(Json(
        update_for(&state.comment_service, &state.user_service, id, payload).await?,
    ))
}
