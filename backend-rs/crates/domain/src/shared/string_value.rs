use crate::shared::error::ValidationError;

/// Trimmed, length-bounded string that rejects empty input after trimming.
///
/// Each per-field wrapper type (e.g. `RegionName`, `Species`) specifies its
/// own `min` / `max` bounds; `NonEmptyString` enforces them in Unicode scalar
/// value counts, not bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(
        value: impl Into<String>,
        field: &'static str,
        min: usize,
        max: usize,
    ) -> Result<Self, ValidationError> {
        let trimmed = value.into().trim().to_string();
        let len = trimmed.chars().count();
        if len == 0 {
            return Err(ValidationError::EmptyString { field });
        }
        if len < min {
            return Err(ValidationError::TooShort {
                field,
                min,
                got: len,
            });
        }
        if len > max {
            return Err(ValidationError::TooLong {
                field,
                max,
                got: len,
            });
        }
        Ok(Self(trimmed))
    }

    pub fn reconstitute(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Generates a `NonEmptyString`-backed newtype with the standard set of
/// constructors (`new`, `reconstitute`), `as_str` accessor, and a `Display`
/// impl. Variable: name, field label, min/max length.
///
/// ```ignore
/// newtype_nonempty! {
///     /// Botanical or common species name, 1–255 characters after trimming.
///     Species, "tree.species", 1, 255
/// }
/// ```
#[macro_export]
macro_rules! newtype_nonempty {
    (
        $(#[$attr:meta])*
        $name:ident, $label:literal, $min:literal, $max:literal $(,)?
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name($crate::shared::string_value::NonEmptyString);

        impl $name {
            pub fn new(
                value: impl Into<String>,
            ) -> ::std::result::Result<Self, $crate::shared::error::ValidationError> {
                Ok(Self($crate::shared::string_value::NonEmptyString::new(
                    value, $label, $min, $max,
                )?))
            }

            pub fn reconstitute(value: String) -> Self {
                Self($crate::shared::string_value::NonEmptyString::reconstitute(value))
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn rejects_empty_string() {
        let err = NonEmptyString::new("", "name", 1, 10).unwrap_err();
        assert_eq!(err, ValidationError::EmptyString { field: "name" });
    }

    #[test]
    fn rejects_whitespace_only() {
        assert_err!(NonEmptyString::new("   ", "name", 1, 10));
    }

    #[test]
    fn trims_input() {
        let v = NonEmptyString::new("  hello  ", "name", 1, 10).unwrap();
        assert_eq!(v.as_str(), "hello");
    }

    #[test]
    fn accepts_within_bounds() {
        assert_ok!(NonEmptyString::new("ab", "name", 2, 10));
        assert_ok!(NonEmptyString::new("abcdefghij", "name", 2, 10));
    }

    #[test]
    fn rejects_below_min() {
        let err = NonEmptyString::new("a", "name", 2, 10).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::TooShort { min: 2, got: 1, .. }
        ));
    }

    #[test]
    fn rejects_above_max() {
        let err = NonEmptyString::new("abcdefghijk", "name", 2, 10).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::TooLong {
                max: 10,
                got: 11,
                ..
            }
        ));
    }

    #[test]
    fn counts_unicode_chars_not_bytes() {
        let v = NonEmptyString::new("äöü", "name", 1, 3).unwrap();
        assert_eq!(v.as_str(), "äöü");
    }
}
