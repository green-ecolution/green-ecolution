use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    comment::{Comment, CommentDraft, CommentSubject, CommentView},
    shared::pagination::{Page, Pagination},
};

/// Read-side access to comments.
#[async_trait]
pub trait CommentReader: Send + Sync {
    /// Returns the aggregate rather than the view: the delete path has to check
    /// the author and the subject before allowing the removal.
    async fn by_id(&self, id: Id<Comment>) -> Result<Comment, RepositoryError>;
    /// Newest first.
    async fn list_for_subject(
        &self,
        subject: CommentSubject,
        pagination: Pagination,
    ) -> Result<Page<CommentView>, RepositoryError>;
}

/// Write-side access to comments.
#[async_trait]
pub trait CommentWriter: Send + Sync {
    async fn save_new(&self, draft: CommentDraft) -> Result<Comment, RepositoryError>;
    async fn save(&self, comment: &Comment) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Comment>) -> Result<(), RepositoryError>;
    /// Removes every comment of a subject. Called when the parent entity is
    /// deleted, since the polymorphic subject rules out a cascading FK.
    async fn delete_for_subject(&self, subject: CommentSubject) -> Result<u64, RepositoryError>;
}
