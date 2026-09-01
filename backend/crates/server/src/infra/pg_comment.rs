use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    comment::{
        Comment, CommentDraft, CommentReader, CommentSnapshot, CommentSubject, CommentView,
        CommentWriter,
    },
    shared::pagination::{Page, Pagination},
};
use uuid::Uuid;

pub struct PgCommentRepository {
    pool: PgPool,
}

impl PgCommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct CommentRow {
    id: Uuid,
    subject_type: String,
    subject_id: Uuid,
    author_id: Uuid,
    body: String,
}

impl TryFrom<CommentRow> for CommentSnapshot {
    type Error = RepositoryError;

    fn try_from(row: CommentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            subject: CommentSubject::from_parts(&row.subject_type, row.subject_id)?,
            author_id: row.author_id,
            body: row.body,
        })
    }
}

fn to_view(row: CommentRow) -> Result<CommentView, RepositoryError> {
    let snapshot = CommentSnapshot::try_from(row)?;
    Ok(CommentView::from(&Comment::reconstitute(snapshot)))
}

#[async_trait::async_trait]
impl CommentReader for PgCommentRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Comment>) -> Result<Comment, RepositoryError> {
        let row = sqlx::query_as!(
            CommentRow,
            r#"SELECT id, subject_type, subject_id, author_id, body
               FROM comments WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Comment::reconstitute(CommentSnapshot::try_from(row)?))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn list_for_subject(
        &self,
        subject: CommentSubject,
        pagination: Pagination,
    ) -> Result<Page<CommentView>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM comments
               WHERE subject_type = $1 AND subject_id = $2"#,
            subject.kind(),
            subject.raw_id(),
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            CommentRow,
            r#"SELECT id, subject_type, subject_id, author_id, body
               FROM comments
               WHERE subject_type = $1 AND subject_id = $2
               ORDER BY id DESC
               LIMIT $3 OFFSET $4"#,
            subject.kind(),
            subject.raw_id(),
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(to_view)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page { items, total })
    }
}

#[async_trait::async_trait]
impl CommentWriter for PgCommentRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: CommentDraft) -> Result<Comment, RepositoryError> {
        let id = Id::<Comment>::new_v7();
        sqlx::query!(
            r#"INSERT INTO comments (id, subject_type, subject_id, author_id, body)
               VALUES ($1, $2, $3, $4, $5)"#,
            id.value(),
            draft.subject.kind(),
            draft.subject.raw_id(),
            draft.author_id,
            draft.body.as_str(),
        )
        .execute(&self.pool)
        .await?;

        Ok(Comment::reconstitute(CommentSnapshot {
            id: id.value(),
            subject: draft.subject,
            author_id: draft.author_id,
            body: draft.body.as_str().to_owned(),
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Comment>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM comments WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete_for_subject(&self, subject: CommentSubject) -> Result<u64, RepositoryError> {
        let result = sqlx::query!(
            r#"DELETE FROM comments WHERE subject_type = $1 AND subject_id = $2"#,
            subject.kind(),
            subject.raw_id(),
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
