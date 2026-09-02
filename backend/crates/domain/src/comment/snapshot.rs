use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::comment::CommentSubject;

/// Raw DB-row mapping used exclusively for aggregate rehydration. The subject
/// is already typed: the adapter parses the two raw columns so `reconstitute`
/// stays infallible.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct CommentSnapshot {
    pub id: Uuid,
    pub subject: CommentSubject,
    pub author_id: Uuid,
    pub body: String,
    pub edited_at: Option<DateTime<Utc>>,
}
