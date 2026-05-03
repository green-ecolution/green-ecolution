//! Shared kernel — value objects and cross-cutting types used by every
//! aggregate.
//!
//! Provides: `ValidationError`, `NonEmptyString`, `Email`, `Coordinate`,
//! `Distance`, `WaterCapacity`, `Provenance`, `Pagination`, `WateringStatus`,
//! and the helper types in `string_value`.

pub mod coordinates;
pub mod distance;
pub mod email;
pub mod error;
pub mod pagination;
pub mod provenance;
pub mod provider_info;
pub mod string_value;
pub mod water_capacity;
pub mod watering_status;
