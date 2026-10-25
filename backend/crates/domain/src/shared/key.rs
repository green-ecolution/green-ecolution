use crate::shared::error::ValidationError;

pub const KEY_MAX_LEN: usize = 32;

/// Stable keys are persisted and referenced across modules, so they share one
/// format: `[a-z][a-z0-9_]*`, at most 32 bytes.
pub fn validate_key(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyString { field });
    }
    if value.len() > KEY_MAX_LEN {
        return Err(ValidationError::TooLong {
            field,
            max: KEY_MAX_LEN,
            got: value.len(),
        });
    }
    let mut bytes = value.bytes();
    let starts_with_letter = bytes.next().is_some_and(|b| b.is_ascii_lowercase());
    let rest_valid = bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !(starts_with_letter && rest_valid) {
        return Err(ValidationError::InvalidFormat {
            field,
            reason: "expected [a-z][a-z0-9_]*".to_string(),
        });
    }
    Ok(())
}
