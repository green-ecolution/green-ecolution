//! Comment aggregate — free-text note someone leaves on a tree cluster or a
//! watering plan.
//!
//! Immutable after creation: there is no edit path, so the aggregate has no
//! mutation methods and emits no domain events. Deletion is a repository
//! operation. The subject is typed rather than a raw pair of columns so that
//! adding a further commentable aggregate is a new enum variant.

pub mod repository;
pub mod snapshot;
pub mod view;

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
        }
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
        });
        assert_eq!(comment.id.value(), id);
        assert_eq!(comment.subject, subject);
        assert_eq!(comment.author_id, author_id);
        assert_eq!(comment.body.as_str(), "Bereits gewässert");
    }
}
