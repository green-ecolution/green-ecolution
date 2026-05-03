use serde_json::Value;

use crate::shared::{error::ValidationError, string_value::NonEmptyString};

/// Opaque provider identifier (e.g. `"tbz"`, `"smarte-grenzregion"`),
/// 1–64 characters after trimming.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(NonEmptyString);

impl ProviderId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(value, "provider", 1, 64)?))
    }

    pub fn reconstitute(value: String) -> Self {
        Self(NonEmptyString::reconstitute(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Attribution metadata attached to most aggregates.
///
/// Tracks which external system imported an entity (`provider`) and carries
/// an opaque JSON payload for provider-specific data (`additional_info`).
/// Both fields are optional; the default is an empty struct.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Provenance {
    provider: Option<ProviderId>,
    additional_info: Option<Value>,
}

impl Provenance {
    pub fn new(provider: Option<ProviderId>, additional_info: Option<Value>) -> Self {
        Self {
            provider,
            additional_info,
        }
    }

    pub fn reconstitute(provider: Option<String>, additional_info: Option<Value>) -> Self {
        Self {
            provider: provider.map(ProviderId::reconstitute),
            additional_info,
        }
    }

    pub fn provider(&self) -> Option<&ProviderId> {
        self.provider.as_ref()
    }

    pub fn additional_info(&self) -> Option<&Value> {
        self.additional_info.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err;
    use serde_json::json;

    #[test]
    fn provider_id_rejects_empty() {
        assert_err!(ProviderId::new(""));
    }

    #[test]
    fn provider_id_rejects_too_long() {
        let s: String = "a".repeat(65);
        assert_err!(ProviderId::new(s));
    }

    #[test]
    fn provider_id_accepts_valid() {
        let p = ProviderId::new("tbz").unwrap();
        assert_eq!(p.as_str(), "tbz");
    }

    #[test]
    fn provenance_default_is_empty() {
        let p = Provenance::default();
        assert!(p.provider().is_none());
        assert!(p.additional_info().is_none());
    }

    #[test]
    fn provenance_carries_values() {
        let p = Provenance::new(
            Some(ProviderId::new("smarte-grenzregion").unwrap()),
            Some(json!({"region": "DE-NW"})),
        );
        assert_eq!(p.provider().unwrap().as_str(), "smarte-grenzregion");
        assert!(p.additional_info().is_some());
    }
}
