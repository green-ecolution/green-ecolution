//! Irrigation module domain: trees, tree clusters, watering status and the
//! soil physics that derives it. Depends on the core `domain` only.

pub mod cluster;
pub mod evaluation;
pub mod events;
pub mod plugin_refs;
pub mod settings;
pub mod tree;
pub mod watering_status;

use domain::{
    capacity::{CapacityKind, CapacityKindSpec},
    module::ModuleKey,
};

pub const KEY: &str = "irrigation";

pub fn key() -> ModuleKey {
    ModuleKey::new(KEY).expect("static key is valid")
}

pub const WATER: &str = "water";

pub fn capacity_kind() -> CapacityKind {
    CapacityKind::new(WATER).expect("static capacity kind is valid")
}

/// Irrigation tours carry water; refilling at a station is optional.
pub fn capacity_kind_spec() -> CapacityKindSpec {
    CapacityKindSpec {
        kind: capacity_kind(),
        unit: "l",
        returns_replenished: false,
    }
}
