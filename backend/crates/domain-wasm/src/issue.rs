use domain::shared::error::ValidationError;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ValidationIssue {
    pub path: String,
    pub field: String,
    pub key: String,
    pub params: Value,
}

impl ValidationIssue {
    /// Convert a domain `ValidationError` into a frontend-friendly issue.
    ///
    /// `path` is the form-field path the frontend uses; `field` keeps the
    /// original namespaced domain field for debugging/logging; `key` is the
    /// i18n key (`{field}.{variant}`); `params` carry the variant payload.
    pub fn from_error(err: &ValidationError, path: impl Into<String>) -> Self {
        let (field, suffix, params) = decompose(err);
        let path = path.into();
        let key = format!("{}.{}", field, suffix);
        Self {
            path,
            field: field.to_string(),
            key,
            params,
        }
    }
}

fn decompose(err: &ValidationError) -> (&'static str, &'static str, Value) {
    match err {
        ValidationError::EmptyString { field } => (field, "empty", json!({})),
        ValidationError::TooShort { field, min, got } => {
            (field, "tooShort", json!({ "min": min, "got": got }))
        }
        ValidationError::TooLong { field, max, got } => {
            (field, "tooLong", json!({ "max": max, "got": got }))
        }
        ValidationError::OutOfRange {
            field,
            min,
            max,
            got,
        } => (
            field,
            "outOfRange",
            json!({ "min": min, "max": max, "got": got }),
        ),
        ValidationError::InvalidFormat { field, reason } => {
            (field, "invalidFormat", json!({ "reason": reason }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_maps_to_empty_suffix() {
        let err = ValidationError::EmptyString {
            field: "tree.species",
        };
        let issue = ValidationIssue::from_error(&err, "species");
        assert_eq!(issue.path, "species");
        assert_eq!(issue.field, "tree.species");
        assert_eq!(issue.key, "tree.species.empty");
        assert_eq!(issue.params, json!({}));
    }

    #[test]
    fn too_short_includes_min_and_got() {
        let err = ValidationError::TooShort {
            field: "cluster.name",
            min: 2,
            got: 1,
        };
        let issue = ValidationIssue::from_error(&err, "name");
        assert_eq!(issue.key, "cluster.name.tooShort");
        assert_eq!(issue.params, json!({ "min": 2, "got": 1 }));
    }

    #[test]
    fn too_long_includes_max_and_got() {
        let err = ValidationError::TooLong {
            field: "cluster.name",
            max: 10,
            got: 12,
        };
        let issue = ValidationIssue::from_error(&err, "name");
        assert_eq!(issue.key, "cluster.name.tooLong");
        assert_eq!(issue.params, json!({ "max": 10, "got": 12 }));
    }

    #[test]
    fn out_of_range_includes_bounds_and_got() {
        let err = ValidationError::OutOfRange {
            field: "coordinate.latitude",
            min: -90.0,
            max: 90.0,
            got: 91.0,
        };
        let issue = ValidationIssue::from_error(&err, "latitude");
        assert_eq!(issue.key, "coordinate.latitude.outOfRange");
        assert_eq!(
            issue.params,
            json!({ "min": -90.0, "max": 90.0, "got": 91.0 })
        );
    }

    #[test]
    fn invalid_format_includes_reason() {
        let err = ValidationError::InvalidFormat {
            field: "user.email",
            reason: "missing @".into(),
        };
        let issue = ValidationIssue::from_error(&err, "email");
        assert_eq!(issue.key, "user.email.invalidFormat");
        assert_eq!(issue.params, json!({ "reason": "missing @" }));
    }
}
