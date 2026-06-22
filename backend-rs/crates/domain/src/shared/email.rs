use crate::shared::error::ValidationError;

/// Trimmed, heuristically validated email address.
///
/// Validation requires a `@`, a domain with at least one `.`, and no
/// whitespace. This is intentionally permissive — it catches obvious typos
/// without pulling in a full RFC 5321 parser.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let trimmed = value.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyString { field: "email" });
        }
        if !looks_like_email(&trimmed) {
            return Err(ValidationError::InvalidFormat {
                field: "email",
                reason: "must be a valid email address".to_string(),
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

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn looks_like_email(s: &str) -> bool {
    let bytes = s.as_bytes();
    let at = match bytes.iter().position(|&b| b == b'@') {
        Some(i) => i,
        None => return false,
    };
    if at == 0 || at == bytes.len() - 1 {
        return false;
    }
    let domain = &s[at + 1..];
    if !domain.contains('.') {
        return false;
    }
    if bytes.iter().any(|&b| b.is_ascii_whitespace()) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn rejects_empty() {
        assert_err!(Email::new(""));
    }

    #[test]
    fn rejects_missing_at() {
        assert_err!(Email::new("foo.bar.com"));
    }

    #[test]
    fn rejects_missing_domain_dot() {
        assert_err!(Email::new("foo@localhost"));
    }

    #[test]
    fn rejects_whitespace() {
        assert_err!(Email::new("foo @bar.com"));
    }

    #[test]
    fn accepts_simple_address() {
        assert_ok!(Email::new("toni.tester@green-ecolution.de"));
    }

    #[test]
    fn trims_input() {
        let e = Email::new("  test@example.com  ").unwrap();
        assert_eq!(e.as_str(), "test@example.com");
    }
}
