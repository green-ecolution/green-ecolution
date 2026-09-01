use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    Id,
    comment::{Comment, CommentSubject},
};

/// Flat read model returned to HTTP handlers. `created_at` comes from the
/// UUID v7 id, so the table needs no timestamp column.
#[derive(Debug, Clone)]
pub struct CommentView {
    pub id: Uuid,
    pub subject: CommentSubject,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Comment> for CommentView {
    fn from(comment: &Comment) -> Self {
        Self {
            id: comment.id.value(),
            subject: comment.subject,
            author_id: comment.author_id,
            body: comment.body.as_str().to_owned(),
            created_at: created_at_of(comment.id),
        }
    }
}

/// Panics only if a `comments.id` was not minted as UUID v7, which the writer
/// guarantees.
fn created_at_of(id: Id<Comment>) -> DateTime<Utc> {
    id.created_at().expect("comments.id is minted as uuid v7")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comment::{CommentBody, CommentSubject};

    #[test]
    fn view_derives_created_at_from_id() {
        let comment = Comment {
            id: Id::new_v7(),
            subject: CommentSubject::TreeCluster(Id::new_v7()),
            author_id: Uuid::new_v4(),
            body: CommentBody::new("Notiz").unwrap(),
        };
        let before = Utc::now();
        let view = CommentView::from(&comment);
        assert_eq!(view.id, comment.id.value());
        assert_eq!(view.body, "Notiz");
        assert!(
            (view.created_at - before).num_seconds().abs() < 5,
            "created_at must come from the v7 timestamp"
        );
    }
}
