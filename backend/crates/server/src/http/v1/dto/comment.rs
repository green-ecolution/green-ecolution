use serde::{Deserialize, Serialize};

use domain::comment::CommentView;

/// A single comment on a tree cluster or a watering plan.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentResponse {
    #[schema(example = "0199a8e9-7c4f-7000-8000-000000000000")]
    pub id: String,
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub author_id: String,
    /// Display name of the author; absent if the account no longer resolves.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "tbz1", nullable)]
    pub author_name: Option<String>,
    #[schema(example = "Boden war noch feucht")]
    pub body: String,
    #[schema(example = "2026-09-01T13:30:00+00:00")]
    pub created_at: String,
    /// Present once the author has edited the comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2026-09-01T14:00:00+00:00", nullable)]
    pub edited_at: Option<String>,
}

impl CommentResponse {
    /// `author_name` is resolved by the handler, which owns the user service;
    /// the comment service deliberately does not depend on it.
    pub fn from_parts(view: &CommentView, author_name: Option<String>) -> Self {
        Self {
            id: view.id.to_string(),
            author_id: view.author_id.to_string(),
            author_name,
            body: view.body.clone(),
            created_at: view.created_at.to_rfc3339(),
            edited_at: view.edited_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Body of the create request. Author and timestamp come from the server.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCommentRequest {
    #[schema(example = "Boden war noch feucht", min_length = 1, max_length = 2000)]
    pub body: String,
}

/// Body of the edit request. A separate type from `CreateCommentRequest` so
/// the generated client does not carry a "Create" type on an update endpoint.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCommentRequest {
    #[schema(
        example = "Boden ist mittlerweile trocken",
        min_length = 1,
        max_length = 2000
    )]
    pub body: String,
}
