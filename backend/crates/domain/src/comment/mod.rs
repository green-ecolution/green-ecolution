//! Comment aggregate — free-text note someone leaves on a tree cluster or a
//! watering plan.
//!
//! The only mutation is the author replacing the text via [`Comment::edit`];
//! there are no other transitions and no subscribers, so the aggregate emits
//! no domain events. Deletion is a repository operation. The subject is typed
//! rather than a raw pair of columns so that adding a further commentable
//! aggregate is a new enum variant.

pub mod repository;
pub mod snapshot;
pub mod view;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    Id, cluster::TreeCluster, shared::error::ValidationError, watering_plan::WateringPlan,
};

pub use repository::{CommentReader, CommentWriter};
#[doc(hidden)]
pub use snapshot::CommentSnapshot;
pub use view::CommentView;

crate::newtype_nonempty! {
    /// Comment text, 1–2000 characters after trimming.
    CommentBody, "comment.body", 1, 2000
}

/// The entity a comment is attached to. Carries the typed id so a cluster id
/// can never be mistaken for a plan id, and a wire value for the DB column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentSubject {
    TreeCluster(Id<TreeCluster>),
    WateringPlan(Id<WateringPlan>),
}

impl CommentSubject {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::TreeCluster(_) => "tree_cluster",
            Self::WateringPlan(_) => "watering_plan",
        }
    }

    pub fn raw_id(&self) -> Uuid {
        match self {
            Self::TreeCluster(id) => id.value(),
            Self::WateringPlan(id) => id.value(),
        }
    }

    pub fn from_parts(kind: &str, id: Uuid) -> Result<Self, ValidationError> {
        match kind {
            "tree_cluster" => Ok(Self::TreeCluster(Id::new(id))),
            "watering_plan" => Ok(Self::WateringPlan(Id::new(id))),
            other => Err(ValidationError::InvalidFormat {
                field: "comment.subject_type",
                reason: format!("unknown subject type '{other}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub id: Id<Comment>,
    pub subject: CommentSubject,
    pub author_id: Uuid,
    pub body: CommentBody,
    /// Set on the aggregate, not just the view: that a text was changed after
    /// the fact is a fact a reader is entitled to, the same reasoning as
    /// `Vehicle::archived_at`, not audit bookkeeping.
    edited_at: Option<DateTime<Utc>>,
}

/// Input for creating a new [`Comment`]. The id and the creation timestamp are
/// minted by the writer.
#[derive(Debug, Clone)]
pub struct CommentDraft {
    pub subject: CommentSubject,
    pub author_id: Uuid,
    pub body: CommentBody,
}

impl Comment {
    #[doc(hidden)]
    pub fn reconstitute(snap: CommentSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            subject: snap.subject,
            author_id: snap.author_id,
            body: CommentBody::reconstitute(snap.body),
            edited_at: snap.edited_at,
        }
    }

    pub fn edited_at(&self) -> Option<DateTime<Utc>> {
        self.edited_at
    }

    /// Replaces the text. Returns whether anything changed, so an unchanged
    /// request does not leave a misleading edit marker.
    pub fn edit(&mut self, body: CommentBody, at: DateTime<Utc>) -> bool {
        if self.body == body {
            return false;
        }
        self.body = body;
        self.edited_at = Some(at);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn body_rejects_empty() {
        assert_err!(CommentBody::new(""));
    }

    #[test]
    fn body_rejects_whitespace_only() {
        assert_err!(CommentBody::new("   "));
    }

    #[test]
    fn body_trims_input() {
        let body = CommentBody::new("  Rasen war trocken  ").unwrap();
        assert_eq!(body.as_str(), "Rasen war trocken");
    }

    #[test]
    fn body_accepts_two_thousand_characters() {
        assert_ok!(CommentBody::new("a".repeat(2000)));
    }

    #[test]
    fn body_rejects_more_than_two_thousand_characters() {
        assert_err!(CommentBody::new("a".repeat(2001)));
    }

    #[test]
    fn cluster_subject_roundtrips_through_parts() {
        let id = Id::<TreeCluster>::new_v7();
        let subject = CommentSubject::TreeCluster(id);
        assert_eq!(subject.kind(), "tree_cluster");
        let back = CommentSubject::from_parts(subject.kind(), subject.raw_id()).unwrap();
        assert_eq!(back, subject);
    }

    #[test]
    fn plan_subject_roundtrips_through_parts() {
        let id = Id::<WateringPlan>::new_v7();
        let subject = CommentSubject::WateringPlan(id);
        assert_eq!(subject.kind(), "watering_plan");
        let back = CommentSubject::from_parts(subject.kind(), subject.raw_id()).unwrap();
        assert_eq!(back, subject);
    }

    #[test]
    fn subject_from_parts_rejects_unknown_kind() {
        assert_err!(CommentSubject::from_parts("tree", Uuid::now_v7()));
    }

    #[test]
    fn subjects_of_different_kinds_with_same_uuid_are_not_equal() {
        let raw = Uuid::now_v7();
        let cluster = CommentSubject::TreeCluster(Id::new(raw));
        let plan = CommentSubject::WateringPlan(Id::new(raw));
        assert_ne!(cluster, plan);
    }

    #[test]
    fn reconstitute_restores_every_field() {
        let id = Uuid::now_v7();
        let subject = CommentSubject::TreeCluster(Id::new_v7());
        let author_id = Uuid::new_v4();
        let comment = Comment::reconstitute(CommentSnapshot {
            id,
            subject,
            author_id,
            body: "Bereits gewässert".to_owned(),
            edited_at: None,
        });
        assert_eq!(comment.id.value(), id);
        assert_eq!(comment.subject, subject);
        assert_eq!(comment.author_id, author_id);
        assert_eq!(comment.body.as_str(), "Bereits gewässert");
        assert_eq!(comment.edited_at(), None);
    }

    #[test]
    fn reconstitute_round_trips_edited_at_when_present() {
        let at = Utc::now();
        let comment = Comment::reconstitute(CommentSnapshot {
            id: Uuid::now_v7(),
            subject: CommentSubject::TreeCluster(Id::new_v7()),
            author_id: Uuid::new_v4(),
            body: "Bereits gewässert".to_owned(),
            edited_at: Some(at),
        });
        assert_eq!(comment.edited_at(), Some(at));
    }

    #[test]
    fn edit_with_different_body_changes_text_and_sets_edited_at() {
        let mut comment = Comment::reconstitute(CommentSnapshot {
            id: Uuid::now_v7(),
            subject: CommentSubject::TreeCluster(Id::new_v7()),
            author_id: Uuid::new_v4(),
            body: "alter Text".to_owned(),
            edited_at: None,
        });
        let at = Utc::now();
        let changed = comment.edit(CommentBody::new("neuer Text").unwrap(), at);
        assert!(changed);
        assert_eq!(comment.body.as_str(), "neuer Text");
        assert_eq!(comment.edited_at(), Some(at));
    }

    #[test]
    fn edit_with_identical_body_returns_false_and_leaves_untouched() {
        let original_edited_at = Utc::now();
        let mut comment = Comment::reconstitute(CommentSnapshot {
            id: Uuid::now_v7(),
            subject: CommentSubject::TreeCluster(Id::new_v7()),
            author_id: Uuid::new_v4(),
            body: "unveraendert".to_owned(),
            edited_at: Some(original_edited_at),
        });
        let changed = comment.edit(CommentBody::new("unveraendert").unwrap(), Utc::now());
        assert!(!changed);
        assert_eq!(comment.body.as_str(), "unveraendert");
        assert_eq!(comment.edited_at(), Some(original_edited_at));
    }
}
