use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::{
    Id,
    comment::{
        Comment, CommentBody, CommentDraft, CommentReader, CommentSubject, CommentView,
        CommentWriter,
    },
    shared::pagination::{Page, Pagination},
};
use uuid::Uuid;

use super::ServiceError;

pub struct CommentService {
    reader: Arc<dyn CommentReader>,
    writer: Arc<dyn CommentWriter>,
}

impl CommentService {
    pub fn new(reader: Arc<dyn CommentReader>, writer: Arc<dyn CommentWriter>) -> Self {
        Self { reader, writer }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(
        &self,
        subject: CommentSubject,
        pagination: Pagination,
    ) -> Result<Page<CommentView>, ServiceError> {
        Ok(self.reader.list_for_subject(subject, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(
        &self,
        subject: CommentSubject,
        author_id: Uuid,
        body: CommentBody,
    ) -> Result<CommentView, ServiceError> {
        let comment = self
            .writer
            .save_new(CommentDraft {
                subject,
                author_id,
                body,
            })
            .await?;
        Ok(CommentView::from(&comment))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_id(&self, id: Id<Comment>) -> Result<Comment, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn delete(&self, id: Id<Comment>) -> Result<(), ServiceError> {
        Ok(self.writer.delete(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update(
        &self,
        id: Id<Comment>,
        body: CommentBody,
        at: DateTime<Utc>,
    ) -> Result<CommentView, ServiceError> {
        let mut comment = self.reader.by_id(id).await?;
        if comment.edit(body, at) {
            self.writer.save(&comment).await?;
        }
        Ok(CommentView::from(&comment))
    }
}
