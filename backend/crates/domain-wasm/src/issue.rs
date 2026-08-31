use domain::shared::error::{ValidationError, ValidationIssue as DomainIssue};
use serde::Serialize;
use serde_json::Value;

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
    /// `field`, `key` and `params` come from [`DomainIssue`], the same
    /// constructor the HTTP layer uses, so a form field validated in the
    /// browser and the same field rejected by the server resolve to one
    /// catalog entry. Only `path` is added here: it is the form-field path,
    /// which exists in the browser and has no meaning server-side.
    pub fn from_error(err: &ValidationError, path: impl Into<String>) -> Self {
        let issue = DomainIssue::from(err);
        Self {
            path: path.into(),
            field: issue.field.to_string(),
            key: issue.key,
            params: issue.params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn the_wasm_issue_agrees_with_the_shared_constructor() {
        let err = ValidationError::TooLong {
            field: "vehicle.model",
            max: 128,
            got: 200,
        };
        let shared = DomainIssue::from(&err);
        let wire = ValidationIssue::from_error(&err, "model");

        assert_eq!(wire.field, shared.field);
        assert_eq!(wire.key, shared.key);
        assert_eq!(wire.params, shared.params);
    }
}
