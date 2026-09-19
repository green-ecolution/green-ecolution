//! Shared machinery for paginated list queries.
//!
//! These queries are assembled at runtime, so they are not covered by sqlx's
//! compile-time query checks. What keeps that safe: values reach SQL only
//! through `push_bind`, and every piece of query text (table, join, column,
//! expression) is a `&'static str` written in this repository, never text
//! that came from a request.

pub mod predicate;

pub use predicate::{ArchiveFilter, Predicate};
