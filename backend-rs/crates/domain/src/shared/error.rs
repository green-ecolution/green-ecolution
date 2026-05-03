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
