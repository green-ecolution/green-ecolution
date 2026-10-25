//! Irrigation module domain: trees, tree clusters, watering status and the
//! soil physics that derives it. Depends on the core `domain` only.

pub mod cluster;
pub mod evaluation;
pub mod events;
pub mod plugin_refs;
pub mod settings;
pub mod tree;
pub mod watering_status;

use domain::module::ModuleKey;

pub const KEY: &str = "irrigation";

pub fn key() -> ModuleKey {
    ModuleKey::new(KEY).expect("static key is valid")
}
