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
                comment::{CommentResponse, CreateCommentRequest, UpdateCommentRequest},
                user::display_name,
            },
            error::ErrorBody,
            pagination::PaginationParams,
            scope,
        },
    },
    service::{
        AuthError, ServiceError, comment_service::CommentService, user_service::UserService,
    },
};
use domain::{
    Id, RepositoryError,
    authorization::{Action, Caller, Permission},
    comment::{Comment, CommentBody, CommentSubject, CommentView},
    shared::pagination::Pagination,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_plan_comments, create_plan_comment))
        .routes(routes!(delete_plan_comment, update_plan_comment))
}

async fn plan_scope(
    state: &AppState,
    caller: Caller,
    plan_id: Uuid,
) -> Result<CommentSubject, ServiceError> {
    let view = state
        .operation_plan_service
        .view_by_id(Id::new(plan_id))
        .await?;
    let resource = state.planning.provider(&view.module)?.plan_resource();
    let ctx = state.authorization_service.context_for(caller).await?;
    scope::ensure_visible(
        &ctx,
        Permission::new(resource, Action::Read),
        view.organization_id,
    )?;
    Ok(CommentSubject::Plan(Id::new(plan_id)))
}

/// Checks that `comment_id` belongs to `subject` and that the caller is its
/// author. The same rule applies to editing and deleting: only the author
/// may act on their own comment, with no fallback via any permission. A
/// comment addressed through the wrong parent reads as 404, never 403.
pub async fn authorize_own_comment(
    comments: &CommentService,
    user_id: Uuid,
    subject: CommentSubject,
    comment_id: Uuid,
) -> Result<Id<Comment>, ServiceError> {
    let id = Id::new(comment_id);
    let comment = comments.by_id(id).await?;
    if comment.subject != subject {
        return Err(RepositoryError::NotFound.into());
    }
    if comment.author_id != user_id {
        return Err(AuthError::Forbidden.into());
    }
    Ok(id)
}

/// Resolves author display names for one page in a single lookup. An IdP
/// outage must not fail the request — the comments themselves are still valid,
/// they just render without a name.
async fn resolve_author_names(users: &UserService, views: &[CommentView]) -> HashMap<Uuid, String> {
    let mut ids: Vec<Uuid> = views.iter().map(|v| v.author_id).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return HashMap::new();
    }
    // One Keycloak round-trip per distinct author, not a batch call. Batching it
    // needs a display-name-only lookup on UserService, deferred to its own change.
    users
        .by_ids(&ids)
        .await
        .inspect_err(|error| tracing::warn!(%error, "failed to resolve comment author names"))
        .unwrap_or_default()
        .iter()
        .map(|user| (user.id, display_name(user)))
        .collect()
}

pub async fn list_for(
    comments: &CommentService,
    users: &UserService,
    subject: CommentSubject,
    params: &PaginationParams,
) -> Result<ListResponse<CommentResponse>, ServiceError> {
    let pagination = Pagination::from(params);
    let page = comments.list(subject, pagination).await?;
    let names = resolve_author_names(users, &page.items).await;
    Ok(ListResponse::from_page_with(
        page,
        &pagination,
        |view: &CommentView| CommentResponse::from_parts(view, names.get(&view.author_id).cloned()),
    ))
}

pub async fn create_for(
    comments: &CommentService,
    users: &UserService,
    subject: CommentSubject,
    author_id: Uuid,
    payload: CreateCommentRequest,
) -> Result<CommentResponse, ServiceError> {
    let body = CommentBody::new(payload.body)?;
    let view = comments.create(subject, author_id, body).await?;
    let names = resolve_author_names(users, std::slice::from_ref(&view)).await;
    Ok(CommentResponse::from_parts(
        &view,
        names.get(&view.author_id).cloned(),
    ))
}

pub async fn update_for(
    comments: &CommentService,
    users: &UserService,
    id: Id<Comment>,
    payload: UpdateCommentRequest,
) -> Result<CommentResponse, ServiceError> {
    let body = CommentBody::new(payload.body)?;
    let view = comments.update(id, body, chrono::Utc::now()).await?;
    let names = resolve_author_names(users, std::slice::from_ref(&view)).await;
    Ok(CommentResponse::from_parts(
        &view,
        names.get(&view.author_id).cloned(),
    ))
}

#[utoipa::path(
    get,
    path = "/plans/{plan_id}/comments",
    tag = "Comments",
    operation_id = "listPlanComments",
    summary = "List comments on a plan",
    description = "Returns the comments of a plan, newest first. Requires read permission on the plan resource of the plan's module in the plan's organization.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID"), PaginationParams),
    responses(
        (status = 200, description = "Paginated list of comments", body = ListResponse<CommentResponse>),
        (status = 404, description = "Plan not found or not visible", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %plan_id))]
pub async fn list_plan_comments(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(plan_id): Path<uuid::Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<CommentResponse>>, ServiceError> {
    let subject = plan_scope(&state, user.caller(), plan_id).await?;
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
    path = "/plans/{plan_id}/comments",
    tag = "Comments",
    operation_id = "createPlanComment",
    summary = "Comment on a plan",
    description = "Adds a comment to a plan. Requires read permission on the plan resource of the plan's module in the plan's organization — anyone who may see the plan may annotate it.",
    params(("plan_id" = uuid::Uuid, Path, description = "Plan ID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = CommentResponse),
        (status = 404, description = "Plan not found or not visible", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %plan_id))]
pub async fn create_plan_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(plan_id): Path<uuid::Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(axum::http::StatusCode, Json<CommentResponse>), ServiceError> {
    let subject = plan_scope(&state, user.caller(), plan_id).await?;
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
    path = "/plans/{plan_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "deletePlanComment",
    summary = "Delete a comment on a plan",
    description = "Removes a comment. Only the comment's author may delete it.",
    params(
        ("plan_id" = uuid::Uuid, Path, description = "Plan ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 403, description = "Not the author", body = ErrorBody),
        (status = 404, description = "Plan or comment not found, or the comment belongs to another plan", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %plan_id, comment.id = %comment_id))]
pub async fn delete_plan_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((plan_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::http::StatusCode, ServiceError> {
    let subject = plan_scope(&state, user.caller(), plan_id).await?;
    let id = authorize_own_comment(&state.comment_service, user.id, subject, comment_id).await?;
    state.comment_service.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/plans/{plan_id}/comments/{comment_id}",
    tag = "Comments",
    operation_id = "updatePlanComment",
    summary = "Edit a comment on a plan",
    description = "Replaces a comment's text. Only the comment's author may edit it.",
    params(
        ("plan_id" = uuid::Uuid, Path, description = "Plan ID"),
        ("comment_id" = uuid::Uuid, Path, description = "Comment ID"),
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = CommentResponse),
        (status = 403, description = "Not the author", body = ErrorBody),
        (status = 404, description = "Plan or comment not found, or the comment belongs to another plan", body = ErrorBody),
        (status = 400, description = "Empty or overlong comment text", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %plan_id, comment.id = %comment_id))]
pub async fn update_plan_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path((plan_id, comment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, ServiceError> {
    let subject = plan_scope(&state, user.caller(), plan_id).await?;
    let id = authorize_own_comment(&state.comment_service, user.id, subject, comment_id).await?;
    Ok(Json(
        update_for(&state.comment_service, &state.user_service, id, payload).await?,
    ))
}
