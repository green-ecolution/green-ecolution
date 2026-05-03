//! WebAssembly bindings for the Green Ecolution domain layer.
//!
//! Thin `#[wasm_bindgen]` wrappers around value-object constructors and
//! per-aggregate draft validators. No business logic lives here.

mod cluster_draft;
mod field_validators;
mod issue;
mod tree_draft;

pub use issue::ValidationIssue;
