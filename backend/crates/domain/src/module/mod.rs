//! Module identity and activation.
//!
//! A module is a feature area such as irrigation or waste collection that an
//! organization switches on or off. Activation inherits down the organization
//! tree the same way operational settings do, including the lock an
//! organization can put on its descendants.

pub mod activation;
pub mod repository;

use std::fmt;

use crate::shared::error::ValidationError;

pub use activation::{ActivationLevel, ActivationResolution, resolve_activation};
pub use repository::{ModuleActivationReader, ModuleActivationWriter};

/// Stable identifier of a module. Once shipped it never changes: stored
/// activations and later plan and sensor references carry it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleKey(String);

impl ModuleKey {
    pub const MAX_LEN: usize = 32;
    const FIELD: &'static str = "module.key";

    pub fn new(value: &str) -> Result<Self, ValidationError> {
        if value.is_empty() {
            return Err(ValidationError::EmptyString { field: Self::FIELD });
        }
        if value.len() > Self::MAX_LEN {
            return Err(ValidationError::TooLong {
                field: Self::FIELD,
                max: Self::MAX_LEN,
                got: value.len(),
            });
        }
        let mut bytes = value.bytes();
        let starts_with_letter = bytes.next().is_some_and(|b| b.is_ascii_lowercase());
        let rest_valid = bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !(starts_with_letter && rest_valid) {
            return Err(ValidationError::InvalidFormat {
                field: Self::FIELD,
                reason: "expected [a-z][a-z0-9_]*".to_string(),
            });
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lowercase_identifiers() {
        assert_eq!(ModuleKey::new("irrigation").unwrap().as_str(), "irrigation");
        assert_eq!(ModuleKey::new("waste_2").unwrap().as_str(), "waste_2");
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!(
            ModuleKey::new(""),
            Err(ValidationError::EmptyString {
                field: "module.key"
            })
        ));
    }

    #[test]
    fn rejects_leading_digit_uppercase_and_dashes() {
        for bad in ["2waste", "Waste", "waste-bins", "_waste", "wäste"] {
            assert!(
                matches!(
                    ModuleKey::new(bad),
                    Err(ValidationError::InvalidFormat {
                        field: "module.key",
                        ..
                    })
                ),
                "{bad} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_overlong_keys() {
        let long = "a".repeat(ModuleKey::MAX_LEN + 1);
        assert!(matches!(
            ModuleKey::new(&long),
            Err(ValidationError::TooLong {
                field: "module.key",
                max: 32,
                got: 33
            })
        ));
        assert!(ModuleKey::new(&"a".repeat(ModuleKey::MAX_LEN)).is_ok());
    }
}
