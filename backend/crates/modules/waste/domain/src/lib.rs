//! Waste module domain: waste bins, their fill level and the collection rules.
//! Depends on the core `domain` only.

pub mod fill;
pub mod settings;
pub mod values;
pub mod waste_bin;

use domain::module::ModuleKey;

pub const KEY: &str = "waste";

pub fn key() -> ModuleKey {
    ModuleKey::new(KEY).expect("static key is valid")
}
