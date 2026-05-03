//! WebAssembly bindings for the Green Ecolution domain layer.
//!
//! Thin `#[wasm_bindgen]` wrappers around value-object constructors and
//! per-aggregate draft validators. No business logic lives here.

mod field_validators;
mod issue;

pub use issue::ValidationIssue;
