//! Tolerant number deserialization for form inputs.
//!
//! HTML number inputs hand React Hook Form raw strings until the user blurs.
//! Strict `f64`/`u32` deserialization would throw a JsError before our
//! validator runs, which RHF surfaces as an unhandled rejection rather than a
//! form error. Instead we accept either a JS number or a string, parse the
//! string with comma→dot tolerance, and surface unparsable input as a
//! [`ValidationIssue`].

use serde::{Deserialize, Deserializer};

use crate::issue::ValidationIssue;

/// Wraps a numeric form field that may arrive as a number or a string.
#[derive(Debug, Clone, Default)]
pub(crate) struct LooseF64(pub Option<f64>);

impl<'de> Deserialize<'de> for LooseF64 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            N(f64),
            S(String),
        }
        Ok(match Repr::deserialize(d)? {
            Repr::N(n) => LooseF64(Some(n)),
            Repr::S(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    LooseF64(None)
                } else {
                    LooseF64(trimmed.replace(',', ".").parse().ok())
                }
            }
        })
    }
}

/// Wraps a numeric form field that must parse as a non-negative integer.
#[derive(Debug, Clone, Default)]
pub(crate) struct LooseU32(pub Option<u32>);

impl<'de> Deserialize<'de> for LooseU32 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            N(i64),
            F(f64),
            S(String),
        }
        Ok(match Repr::deserialize(d)? {
            Repr::N(n) if n >= 0 && n <= u32::MAX as i64 => LooseU32(Some(n as u32)),
            Repr::N(_) => LooseU32(None),
            Repr::F(n) if n.is_finite() && n.fract() == 0.0 && n >= 0.0 && n <= u32::MAX as f64 => {
                LooseU32(Some(n as u32))
            }
            Repr::F(_) => LooseU32(None),
            Repr::S(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    LooseU32(None)
                } else {
                    LooseU32(trimmed.parse().ok())
                }
            }
        })
    }
}

/// Build a `ValidationIssue` for a numeric field whose input was unparsable.
pub(crate) fn invalid_number_issue(field: &'static str, path: &'static str) -> ValidationIssue {
    use domain::shared::error::ValidationError;
    ValidationIssue::from_error(
        &ValidationError::InvalidFormat {
            field,
            reason: "expected a number".into(),
        },
        path,
    )
}

/// Validate that `raw` is a valid wire-format value of the domain enum `T`.
///
/// The domain enum's `serde::Deserialize` impl is the single source of truth
/// for valid variants and their wire spelling. Returns `None` on success, or
/// a `ValidationIssue` whose `key` is `{field}.invalidFormat` on failure.
pub(crate) fn validate_enum<T: serde::de::DeserializeOwned>(
    raw: &str,
    field: &'static str,
    path: &'static str,
) -> Option<ValidationIssue> {
    use domain::shared::error::ValidationError;
    match serde_json::from_value::<T>(serde_json::Value::String(raw.to_string())) {
        Ok(_) => None,
        Err(_) => Some(ValidationIssue::from_error(
            &ValidationError::InvalidFormat {
                field,
                reason: format!("`{}` is not a valid value", raw),
            },
            path,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_loose_f64(value: serde_json::Value) -> LooseF64 {
        serde_json::from_value(value).unwrap()
    }

    fn parse_loose_u32(value: serde_json::Value) -> LooseU32 {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn loose_f64_from_number() {
        assert_eq!(parse_loose_f64(json!(2.5)).0, Some(2.5));
    }

    #[test]
    fn loose_f64_from_numeric_string() {
        assert_eq!(parse_loose_f64(json!("2.5")).0, Some(2.5));
    }

    #[test]
    fn loose_f64_translates_comma() {
        assert_eq!(parse_loose_f64(json!("2,5")).0, Some(2.5));
    }

    #[test]
    fn loose_f64_rejects_garbage() {
        assert_eq!(parse_loose_f64(json!("6d")).0, None);
        assert_eq!(parse_loose_f64(json!("2fooo")).0, None);
    }

    #[test]
    fn loose_f64_treats_empty_as_missing() {
        assert_eq!(parse_loose_f64(json!("")).0, None);
        assert_eq!(parse_loose_f64(json!("   ")).0, None);
    }

    #[test]
    fn loose_u32_from_number() {
        assert_eq!(parse_loose_u32(json!(2024)).0, Some(2024));
    }

    #[test]
    fn loose_u32_from_string() {
        assert_eq!(parse_loose_u32(json!("2024")).0, Some(2024));
    }

    #[test]
    fn loose_u32_rejects_negative() {
        assert_eq!(parse_loose_u32(json!(-1)).0, None);
    }

    #[test]
    fn loose_u32_rejects_fractional() {
        assert_eq!(parse_loose_u32(json!(2024.5)).0, None);
    }

    #[test]
    fn loose_u32_rejects_garbage() {
        assert_eq!(parse_loose_u32(json!("abc")).0, None);
    }
}
