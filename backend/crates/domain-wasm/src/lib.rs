//! WebAssembly bindings for the Green Ecolution domain layer.
//!
//! Thin `#[wasm_bindgen]` wrappers around value-object constructors and
//! per-aggregate draft validators. No business logic lives here.

mod coerce;
mod field_validators;
mod irrigation;
mod issue;
mod plan_draft;
mod vehicle_draft;

pub use issue::ValidationIssue;
