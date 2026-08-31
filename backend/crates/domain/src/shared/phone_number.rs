use crate::shared::error::ValidationError;

/// Trimmed, heuristically validated phone number.
///
/// Allowed characters are digits, space, `-`, `/`, `(`, `)`, and a `+` only
/// as the first character; at least 6 digits must be present. This is
/// intentionally permissive, mirroring `Email` — it catches obvious mistakes
/// (letters, no digits at all) without pulling in a full phone-number parser
/// that would need to know every country's dialing conventions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let trimmed = value.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyString {
                field: "user.phone_number",
            });
        }
        if !looks_like_phone_number(&trimmed) {
            return Err(ValidationError::InvalidFormat {
                field: "user.phone_number",
                reason: "must contain only digits, spaces, -, /, (, ) and an optional leading +, with at least 6 digits".to_string(),
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
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn looks_like_phone_number(s: &str) -> bool {
    let mut digit_count = 0usize;
    for (i, c) in s.chars().enumerate() {
        match c {
            '0'..='9' => digit_count += 1,
            ' ' | '-' | '/' | '(' | ')' => {}
            '+' if i == 0 => {}
            _ => return false,
        }
    }
    digit_count >= 6
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn accepts_international_format() {
        assert_ok!(PhoneNumber::new("+49 461 123456"));
    }

    #[test]
    fn accepts_local_format_with_space() {
        assert_ok!(PhoneNumber::new("0461 123456"));
    }

    #[test]
    fn accepts_local_format_with_dash() {
        assert_ok!(PhoneNumber::new("0461 123-456"));
    }

    #[test]
    fn accepts_slash_separator() {
        assert_ok!(PhoneNumber::new("0461/123456"));
    }

    #[test]
    fn accepts_parentheses_area_code() {
        assert_ok!(PhoneNumber::new("(0461) 123 456"));
    }

    #[test]
    fn accepts_leading_plus_with_parenthesized_zero() {
        assert_ok!(PhoneNumber::new("+49 (0) 461 123456"));
    }

    #[test]
    fn rejects_no_digits_at_all() {
        assert_err!(PhoneNumber::new("lkjlkjdfs"));
    }

    #[test]
    fn rejects_letters_mixed_with_digits() {
        assert_err!(PhoneNumber::new("0461 abc"));
    }

    #[test]
    fn rejects_fewer_than_six_digits() {
        assert_err!(PhoneNumber::new("12345"));
    }

    #[test]
    fn rejects_bare_plus() {
        assert_err!(PhoneNumber::new("+"));
    }

    #[test]
    fn rejects_plus_not_in_leading_position() {
        assert_err!(PhoneNumber::new("49+461123456"));
    }

    #[test]
    fn trims_leading_and_trailing_whitespace() {
        let p = PhoneNumber::new("  0461 123456  ").unwrap();
        assert_eq!(p.as_str(), "0461 123456");
    }
}
