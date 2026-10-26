use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::{
    Id,
    comment::{
        Comment, CommentBody, CommentDraft, CommentReader, CommentSubject, CommentView,
        CommentWriter,
    },
    module::ModuleActivationReader,
    shared::pagination::{Page, Pagination},
    target::{TargetPurpose, TargetRef},
};
use uuid::Uuid;

use super::{
    ServiceError,
    targets::{Admitted, TargetRegistry, ensure_module_active},
};

pub struct CommentService {
    reader: Arc<dyn CommentReader>,
    writer: Arc<dyn CommentWriter>,
    targets: Arc<TargetRegistry>,
    activation: Arc<dyn ModuleActivationReader>,
}

impl CommentService {
    pub fn new(
        reader: Arc<dyn CommentReader>,
        writer: Arc<dyn CommentWriter>,
        targets: Arc<TargetRegistry>,
        activation: Arc<dyn ModuleActivationReader>,
    ) -> Self {
        Self {
            reader,
            writer,
            targets,
            activation,
        }
    }

    /// The target must exist, be commentable and belong to a module that is
    /// active in its organization.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn admit_target(&self, target: &TargetRef) -> Result<Admitted, ServiceError> {
        let admitted = self.targets.admit(target, TargetPurpose::Comment).await?;
        ensure_module_active(
            self.activation.as_ref(),
            &target.module,
            admitted.organization_id,
        )
        .await?;
        Ok(admitted)
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
        if let CommentSubject::Target(target) = &subject {
            self.admit_target(target).await?;
        }
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

    /// Comments hang on a target without a cascading key, so whoever deletes
    /// the target removes its comments through this. Returns how many went.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn delete_for_target(&self, target: &TargetRef) -> Result<u64, ServiceError> {
        Ok(self
            .writer
            .delete_for_subject(CommentSubject::Target(target.clone()))
            .await?)
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
