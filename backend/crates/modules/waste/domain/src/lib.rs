//! Waste module domain: waste bins, their fill level and the collection rules.
//! Depends on the core `domain` only.

pub mod fill;
pub mod settings;
pub mod values;
pub mod waste_bin;

use domain::{
    capacity::{CapacityKind, CapacityKindSpec},
    module::ModuleKey,
};

pub const KEY: &str = "waste";

pub fn key() -> ModuleKey {
    ModuleKey::new(KEY).expect("static key is valid")
}

pub const CARGO_VOLUME: &str = "cargo_volume";

pub fn capacity_kind() -> CapacityKind {
    CapacityKind::new(CARGO_VOLUME).expect("static capacity kind is valid")
}

/// Collection tours fill their cargo space and must unload before they return.
pub fn capacity_kind_spec() -> CapacityKindSpec {
    CapacityKindSpec {
        kind: capacity_kind(),
        unit: "l",
        returns_replenished: true,
    }
}
