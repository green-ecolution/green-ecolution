use secrecy::{ExposeSecret, SecretString};

use super::{DataExportError, ExportUrl};
use crate::shared::error::ValidationError;

pub const MAX_SECRET_LEN: usize = 1024;

crate::newtype_nonempty! {
    /// Username for basic auth or client id for OAuth2.
    ExportPrincipal, "data_export.auth.principal", 1, 256
}

crate::newtype_nonempty! {
    ExportScope, "data_export.auth.scope", 1, 256
}

/// Password or client secret. `Debug` comes from `SecretString` and redacts.
#[derive(Debug, Clone)]
pub struct ExportSecret(SecretString);

impl ExportSecret {
    pub fn new(field: &'static str, value: SecretString) -> Result<Self, ValidationError> {
        let len = value.expose_secret().chars().count();
        if len == 0 {
            return Err(ValidationError::EmptyString { field });
        }
        if len > MAX_SECRET_LEN {
            return Err(ValidationError::TooLong {
                field,
                max: MAX_SECRET_LEN,
                got: len,
            });
        }
        Ok(Self(value))
    }

    #[doc(hidden)]
    pub fn reconstitute(value: String) -> Self {
        Self(SecretString::from(value))
    }

    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportAuthKind {
    None,
    Basic,
    OAuth2ClientCredentials,
}

impl ExportAuthKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Basic => "basic",
            Self::OAuth2ClientCredentials => "oauth2_client_credentials",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "none" => Some(Self::None),
            "basic" => Some(Self::Basic),
            "oauth2_client_credentials" => Some(Self::OAuth2ClientCredentials),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExportAuth {
    None,
    Basic {
        username: ExportPrincipal,
        password: ExportSecret,
    },
    OAuth2ClientCredentials {
        token_url: ExportUrl,
        client_id: ExportPrincipal,
        client_secret: ExportSecret,
        scope: Option<ExportScope>,
    },
}

impl ExportAuth {
    pub fn kind(&self) -> ExportAuthKind {
        match self {
            Self::None => ExportAuthKind::None,
            Self::Basic { .. } => ExportAuthKind::Basic,
            Self::OAuth2ClientCredentials { .. } => ExportAuthKind::OAuth2ClientCredentials,
        }
    }

    pub fn secret(&self) -> Option<&ExportSecret> {
        match self {
            Self::None => None,
            Self::Basic { password, .. } => Some(password),
            Self::OAuth2ClientCredentials { client_secret, .. } => Some(client_secret),
        }
    }

    pub fn principal(&self) -> Option<&ExportPrincipal> {
        match self {
            Self::None => None,
            Self::Basic { username, .. } => Some(username),
            Self::OAuth2ClientCredentials { client_id, .. } => Some(client_id),
        }
    }

    pub fn token_url(&self) -> Option<&ExportUrl> {
        match self {
            Self::OAuth2ClientCredentials { token_url, .. } => Some(token_url),
            _ => None,
        }
    }
}

/// The auth part of a write. The secret is optional because the API never
/// hands it back, so a client resubmitting the form cannot send it again.
#[derive(Debug, Clone)]
pub enum ExportAuthDraft {
    None,
    Basic {
        username: ExportPrincipal,
        password: Option<ExportSecret>,
    },
    OAuth2ClientCredentials {
        token_url: ExportUrl,
        client_id: ExportPrincipal,
        client_secret: Option<ExportSecret>,
        scope: Option<ExportScope>,
    },
}

impl ExportAuthDraft {
    pub fn kind(&self) -> ExportAuthKind {
        match self {
            Self::None => ExportAuthKind::None,
            Self::Basic { .. } => ExportAuthKind::Basic,
            Self::OAuth2ClientCredentials { .. } => ExportAuthKind::OAuth2ClientCredentials,
        }
    }

    pub fn token_url(&self) -> Option<&ExportUrl> {
        match self {
            Self::OAuth2ClientCredentials { token_url, .. } => Some(token_url),
            _ => None,
        }
    }

    /// An omitted secret keeps the stored one, but only while the kind stays
    /// the same: a basic password must never become an OAuth2 client secret.
    pub fn resolve(self, stored: Option<&ExportAuth>) -> Result<ExportAuth, DataExportError> {
        let kind = self.kind();
        let kept = stored
            .filter(|s| s.kind() == kind)
            .and_then(ExportAuth::secret)
            .cloned();
        let missing = DataExportError::SecretMissing {
            kind: kind.as_str(),
        };
        match self {
            Self::None => Ok(ExportAuth::None),
            Self::Basic { username, password } => Ok(ExportAuth::Basic {
                username,
                password: password.or(kept).ok_or(missing)?,
            }),
            Self::OAuth2ClientCredentials {
                token_url,
                client_id,
                client_secret,
                scope,
            } => Ok(ExportAuth::OAuth2ClientCredentials {
                token_url,
                client_id,
                client_secret: client_secret.or(kept).ok_or(missing)?,
                scope,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_export::{DataExportError, ExportUrl, TOKEN_URL_FIELD};
    use secrecy::SecretString;

    fn secret(v: &str) -> ExportSecret {
        ExportSecret::new(
            "data_export.auth.password",
            SecretString::from(v.to_string()),
        )
        .unwrap()
    }

    fn basic(password: Option<&str>) -> ExportAuthDraft {
        ExportAuthDraft::Basic {
            username: ExportPrincipal::new("ge").unwrap(),
            password: password.map(secret),
        }
    }

    fn oauth(secret_value: Option<&str>) -> ExportAuthDraft {
        ExportAuthDraft::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, "https://auth.example.org/token").unwrap(),
            client_id: ExportPrincipal::new("green-ecolution").unwrap(),
            client_secret: secret_value.map(secret),
            scope: None,
        }
    }

    #[test]
    fn a_given_secret_wins() {
        let auth = basic(Some("new")).resolve(None).unwrap();
        assert_eq!(auth.secret().unwrap().expose(), "new");
    }

    #[test]
    fn an_omitted_secret_keeps_the_stored_one_of_the_same_kind() {
        let stored = basic(Some("old")).resolve(None).unwrap();
        let auth = basic(None).resolve(Some(&stored)).unwrap();
        assert_eq!(auth.secret().unwrap().expose(), "old");
    }

    #[test]
    fn switching_the_kind_needs_a_new_secret() {
        let stored = basic(Some("old")).resolve(None).unwrap();
        assert_eq!(
            oauth(None).resolve(Some(&stored)).unwrap_err(),
            DataExportError::SecretMissing {
                kind: "oauth2_client_credentials"
            }
        );
    }

    #[test]
    fn a_first_save_without_secret_is_incomplete() {
        assert_eq!(
            basic(None).resolve(None).unwrap_err(),
            DataExportError::SecretMissing { kind: "basic" }
        );
    }

    #[test]
    fn none_needs_nothing() {
        assert!(matches!(
            ExportAuthDraft::None.resolve(None),
            Ok(ExportAuth::None)
        ));
    }

    #[test]
    fn secrets_are_length_checked_and_never_printed() {
        assert!(ExportSecret::new("f", SecretString::from(String::new())).is_err());
        assert!(ExportSecret::new("f", SecretString::from("x".repeat(1025))).is_err());
        let auth = basic(Some("hunter2")).resolve(None).unwrap();
        assert!(!format!("{auth:?}").contains("hunter2"));
    }

    #[test]
    fn kinds_round_trip_through_their_wire_names() {
        for kind in [
            ExportAuthKind::None,
            ExportAuthKind::Basic,
            ExportAuthKind::OAuth2ClientCredentials,
        ] {
            assert_eq!(ExportAuthKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(ExportAuthKind::parse("bearer"), None);
    }
}
