use secrecy::{ExposeSecret, SecretString};

use super::{DataExportError, ExportTargetConfig, ExportUrl};
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

    /// An omitted secret keeps the stored one only while the kind and every
    /// origin that receives it stay the same: a basic password must never
    /// become an OAuth2 client secret, nor follow a changed host.
    pub fn resolve(
        self,
        base_url: &ExportUrl,
        stored: Option<&ExportTargetConfig>,
    ) -> Result<ExportAuth, DataExportError> {
        let kind = self.kind();
        let same_token_origin = match (stored.and_then(|s| s.auth.token_url()), self.token_url()) {
            (Some(old), Some(new)) => old.same_origin(new),
            (None, None) => true,
            _ => false,
        };
        let kept = stored
            .filter(|s| {
                s.auth.kind() == kind && same_token_origin && s.base_url.same_origin(base_url)
            })
            .and_then(|s| s.auth.secret())
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
    use crate::data_export::{BASE_URL_FIELD, DataExportError, ExportUrl, TOKEN_URL_FIELD};
    use secrecy::SecretString;

    const BASE: &str = "https://frost.example.org/FROST-Server/v1.1";
    const TOKEN: &str = "https://auth.example.org/token";

    fn secret(v: &str) -> ExportSecret {
        ExportSecret::new(
            "data_export.auth.password",
            SecretString::from(v.to_string()),
        )
        .unwrap()
    }

    fn base(raw: &str) -> ExportUrl {
        ExportUrl::new(BASE_URL_FIELD, raw).unwrap()
    }

    fn basic(password: Option<&str>) -> ExportAuthDraft {
        ExportAuthDraft::Basic {
            username: ExportPrincipal::new("ge").unwrap(),
            password: password.map(secret),
        }
    }

    fn oauth_at(token_url: &str, secret_value: Option<&str>) -> ExportAuthDraft {
        ExportAuthDraft::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, token_url).unwrap(),
            client_id: ExportPrincipal::new("green-ecolution").unwrap(),
            client_secret: secret_value.map(secret),
            scope: None,
        }
    }

    fn oauth(secret_value: Option<&str>) -> ExportAuthDraft {
        oauth_at(TOKEN, secret_value)
    }

    fn stored(base_url: &str, draft: ExportAuthDraft) -> ExportTargetConfig {
        let base_url = base(base_url);
        let auth = draft.resolve(&base_url, None).unwrap();
        ExportTargetConfig {
            enabled: true,
            base_url,
            auth,
        }
    }

    #[test]
    fn a_given_secret_wins() {
        let auth = basic(Some("new")).resolve(&base(BASE), None).unwrap();
        assert_eq!(auth.secret().unwrap().expose(), "new");
    }

    #[test]
    fn an_omitted_secret_keeps_the_stored_one_of_the_same_kind() {
        let stored = stored(BASE, basic(Some("old")));
        let auth = basic(None).resolve(&base(BASE), Some(&stored)).unwrap();
        assert_eq!(auth.secret().unwrap().expose(), "old");
    }

    #[test]
    fn a_path_change_on_the_same_origin_keeps_the_secret() {
        let stored = stored(BASE, basic(Some("old")));
        let auth = basic(None)
            .resolve(&base("https://frost.example.org/v1.1"), Some(&stored))
            .unwrap();
        assert_eq!(auth.secret().unwrap().expose(), "old");
    }

    #[test]
    fn a_basic_secret_does_not_follow_a_changed_origin() {
        let stored = stored(BASE, basic(Some("old")));
        for raw in [
            "https://other.example.org/FROST-Server/v1.1",
            "https://frost.example.org:8443/FROST-Server/v1.1",
            "http://frost.example.org/FROST-Server/v1.1",
        ] {
            assert_eq!(
                basic(None).resolve(&base(raw), Some(&stored)).unwrap_err(),
                DataExportError::SecretMissing { kind: "basic" },
                "{raw} kept the secret"
            );
        }
    }

    #[test]
    fn an_explicit_default_port_is_the_same_origin() {
        let stored = stored(BASE, basic(Some("old")));
        assert!(
            basic(None)
                .resolve(
                    &base("https://frost.example.org:443/FROST-Server/v1.1"),
                    Some(&stored)
                )
                .is_ok()
        );
    }

    #[test]
    fn an_oauth2_secret_needs_both_origins_unchanged() {
        let stored = stored(BASE, oauth(Some("old")));

        let kept = oauth(None).resolve(&base(BASE), Some(&stored)).unwrap();
        assert_eq!(kept.secret().unwrap().expose(), "old");

        let missing = DataExportError::SecretMissing {
            kind: "oauth2_client_credentials",
        };
        assert_eq!(
            oauth_at("https://auth.evil.test/token", None)
                .resolve(&base(BASE), Some(&stored))
                .unwrap_err(),
            missing
        );
        assert_eq!(
            oauth(None)
                .resolve(&base("https://other.example.org/v1.1"), Some(&stored))
                .unwrap_err(),
            missing
        );
    }

    #[test]
    fn switching_the_kind_needs_a_new_secret() {
        let stored = stored(BASE, basic(Some("old")));
        assert_eq!(
            oauth(None).resolve(&base(BASE), Some(&stored)).unwrap_err(),
            DataExportError::SecretMissing {
                kind: "oauth2_client_credentials"
            }
        );
    }

    #[test]
    fn a_first_save_without_secret_is_incomplete() {
        assert_eq!(
            basic(None).resolve(&base(BASE), None).unwrap_err(),
            DataExportError::SecretMissing { kind: "basic" }
        );
    }

    #[test]
    fn none_needs_nothing() {
        assert!(matches!(
            ExportAuthDraft::None.resolve(&base(BASE), None),
            Ok(ExportAuth::None)
        ));
    }

    #[test]
    fn secrets_are_length_checked_and_never_printed() {
        assert!(ExportSecret::new("f", SecretString::from(String::new())).is_err());
        assert!(ExportSecret::new("f", SecretString::from("x".repeat(1025))).is_err());
        let auth = basic(Some("hunter2")).resolve(&base(BASE), None).unwrap();
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
