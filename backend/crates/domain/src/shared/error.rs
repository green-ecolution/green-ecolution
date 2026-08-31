use thiserror::Error;

/// Validation failure returned by value-object constructors.
///
/// Every `::new` constructor on a domain value object (e.g. `NonEmptyString`,
/// `Coordinate`) returns this error type on bad input. It converts into
/// [`crate::RepositoryError::DataIntegrity`] so infrastructure code
/// can propagate it without knowing the specific variant.
#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("{field} is empty")]
    EmptyString { field: &'static str },
    #[error("{field} length {got} exceeds max {max}")]
    TooLong {
        field: &'static str,
        max: usize,
        got: usize,
    },
    #[error("{field} length {got} below min {min}")]
    TooShort {
        field: &'static str,
        min: usize,
        got: usize,
    },
    #[error("{field} value {got} out of range [{min}, {max}]")]
    OutOfRange {
        field: &'static str,
        min: f64,
        max: f64,
        got: f64,
    },
    #[error("{field} has invalid format: {reason}")]
    InvalidFormat { field: &'static str, reason: String },
}

/// A validation failure in the shape a client can translate.
///
/// Both delivery paths build this from the same [`ValidationError`]: the WASM
/// bindings for in-browser form validation and the HTTP layer for rules that
/// only the server can check. Keeping one constructor is what stops the two
/// key spaces from drifting apart, which would silently strand catalog
/// entries on one side.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ValidationIssue {
    /// Namespaced domain field label, e.g. `cluster.name`.
    pub field: &'static str,
    /// Translation key, `{field}.{violated rule}`.
    pub key: String,
    /// Values the translated sentence interpolates.
    pub params: serde_json::Value,
}

impl From<&ValidationError> for ValidationIssue {
    fn from(error: &ValidationError) -> Self {
        use serde_json::json;

        let (field, rule, params) = match error {
            ValidationError::EmptyString { field } => (*field, "empty", json!({})),
            ValidationError::TooShort { field, min, got } => {
                (*field, "tooShort", json!({ "min": min, "got": got }))
            }
            ValidationError::TooLong { field, max, got } => {
                (*field, "tooLong", json!({ "max": max, "got": got }))
            }
            ValidationError::OutOfRange {
                field,
                min,
                max,
                got,
            } => (
                *field,
                "outOfRange",
                json!({ "min": min, "max": max, "got": got }),
            ),
            ValidationError::InvalidFormat { field, reason } => {
                (*field, "invalidFormat", json!({ "reason": reason }))
            }
        };

        Self {
            field,
            key: format!("{field}.{rule}"),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_for_empty_string() {
        let err = ValidationError::EmptyString { field: "name" };
        assert_eq!(err.to_string(), "name is empty");
    }

    #[test]
    fn display_for_too_long() {
        let err = ValidationError::TooLong {
            field: "name",
            max: 10,
            got: 12,
        };
        assert_eq!(err.to_string(), "name length 12 exceeds max 10");
    }

    #[test]
    fn display_for_out_of_range() {
        let err = ValidationError::OutOfRange {
            field: "moisture",
            min: 0.0,
            max: 1.0,
            got: 1.5,
        };
        assert_eq!(err.to_string(), "moisture value 1.5 out of range [0, 1]");
    }
}

#[cfg(test)]
mod issue_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_is_the_field_label_plus_the_violated_rule() {
        let issue = ValidationIssue::from(&ValidationError::TooLong {
            field: "cluster.name",
            max: 255,
            got: 300,
        });
        assert_eq!(issue.field, "cluster.name");
        assert_eq!(issue.key, "cluster.name.tooLong");
        assert_eq!(issue.params, json!({ "max": 255, "got": 300 }));
    }

    #[test]
    fn an_empty_field_carries_no_parameters() {
        let issue = ValidationIssue::from(&ValidationError::EmptyString {
            field: "tree.species",
        });
        assert_eq!(issue.key, "tree.species.empty");
        assert_eq!(issue.params, json!({}));
    }

    #[test]
    fn a_range_violation_carries_the_bounds_and_the_offending_value() {
        let issue = ValidationIssue::from(&ValidationError::OutOfRange {
            field: "coordinate.latitude",
            min: -90.0,
            max: 90.0,
            got: 91.0,
        });
        assert_eq!(issue.key, "coordinate.latitude.outOfRange");
        assert_eq!(
            issue.params,
            json!({ "min": -90.0, "max": 90.0, "got": 91.0 })
        );
    }

    #[test]
    fn a_format_violation_carries_its_reason() {
        let issue = ValidationIssue::from(&ValidationError::InvalidFormat {
            field: "user.email",
            reason: "missing @".into(),
        });
        assert_eq!(issue.key, "user.email.invalidFormat");
        assert_eq!(issue.params, json!({ "reason": "missing @" }));
    }

    #[test]
    fn a_too_short_field_carries_the_minimum() {
        let issue = ValidationIssue::from(&ValidationError::TooShort {
            field: "role.name",
            min: 2,
            got: 1,
        });
        assert_eq!(issue.key, "role.name.tooShort");
        assert_eq!(issue.params, json!({ "min": 2, "got": 1 }));
    }
}
