use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::{configuration::AuthSettings, domain::RepositoryError};

// Endpoints are derived from the issuer URL rather than the OIDC discovery
// document — Keycloak's path scheme is stable enough to skip the extra hop.
#[derive(Debug)]
pub struct KeycloakClient {
    http: reqwest::Client,
    issuer_url: Url,
    server_root: Url,
    realm: String,
    pub frontend_client_id: String,
    pub backend_client_id: String,
    pub backend_client_secret: SecretString,
}

impl KeycloakClient {
    pub fn new(settings: &AuthSettings) -> Result<Self, RepositoryError> {
        let issuer_url = Url::parse(&settings.issuer_url)
            .map_err(|e| RepositoryError::Internal(format!("invalid issuer_url: {e}")))?;

        let realm = issuer_url
            .path_segments()
            .and_then(|mut segs| {
                let kind = segs.next()?;
                let name = segs.next()?;
                if kind == "realms" && !name.is_empty() {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                RepositoryError::Internal(format!(
                    "issuer_url {issuer_url} does not match the Keycloak `/realms/<name>` pattern"
                ))
            })?;

        let mut server_root = issuer_url.clone();
        server_root.set_path("/");

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| RepositoryError::Internal(format!("failed to build http client: {e}")))?;

        Ok(Self {
            http,
            issuer_url,
            server_root,
            realm,
            frontend_client_id: settings.frontend_client_id.clone(),
            backend_client_id: settings.backend_client_id.clone(),
            backend_client_secret: settings.backend_client_secret.clone(),
        })
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn issuer_url(&self) -> &Url {
        &self.issuer_url
    }

    pub fn realm(&self) -> &str {
        &self.realm
    }

    pub fn auth_url(&self) -> Url {
        self.realm_oidc("auth")
    }

    pub fn token_url(&self) -> Url {
        self.realm_oidc("token")
    }

    pub fn logout_url(&self) -> Url {
        self.realm_oidc("logout")
    }

    pub fn revoke_url(&self) -> Url {
        self.realm_oidc("revoke")
    }

    pub fn introspect_url(&self) -> Url {
        self.realm_oidc("token/introspect")
    }

    pub fn jwks_url(&self) -> Url {
        self.realm_oidc("certs")
    }

    pub fn admin_users_url(&self) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!("/admin/realms/{}/users", self.realm));
        url
    }

    pub fn admin_user_url(&self, id: &str) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!("/admin/realms/{}/users/{}", self.realm, id));
        url
    }

    pub fn admin_user_reset_password_url(&self, id: &str) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!(
            "/admin/realms/{}/users/{}/reset-password",
            self.realm, id
        ));
        url
    }

    pub fn admin_user_realm_role_mappings_url(&self, id: &str) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!(
            "/admin/realms/{}/users/{}/role-mappings/realm",
            self.realm, id
        ));
        url
    }

    pub fn admin_realm_roles_url(&self) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!("/admin/realms/{}/roles", self.realm));
        url
    }

    pub fn backend_client_secret(&self) -> &str {
        self.backend_client_secret.expose_secret()
    }

    fn realm_oidc(&self, leaf: &str) -> Url {
        let mut url = self.server_root.clone();
        url.set_path(&format!(
            "/realms/{}/protocol/openid-connect/{leaf}",
            self.realm
        ));
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    fn settings(issuer: &str) -> AuthSettings {
        AuthSettings {
            enabled: true,
            issuer_url: issuer.to_string(),
            frontend_client_id: "ui".into(),
            backend_client_id: "backend".into(),
            backend_client_secret: SecretString::from("s".to_string()),
            jwks_refresh_interval_secs: 60,
            jwks_refresh_timeout_secs: 5,
            default_redirect_url: "http://localhost/cb".into(),
            expected_audience: None,
        }
    }

    #[test]
    fn builds_oidc_urls_from_issuer() {
        let kc = KeycloakClient::new(&settings("https://kc.example.com/realms/green")).unwrap();
        assert_eq!(kc.realm(), "green");
        assert_eq!(
            kc.token_url().as_str(),
            "https://kc.example.com/realms/green/protocol/openid-connect/token"
        );
        assert_eq!(
            kc.jwks_url().as_str(),
            "https://kc.example.com/realms/green/protocol/openid-connect/certs"
        );
        assert_eq!(
            kc.admin_users_url().as_str(),
            "https://kc.example.com/admin/realms/green/users"
        );
    }

    #[test]
    fn rejects_non_realm_issuer() {
        let err = KeycloakClient::new(&settings("https://kc.example.com/auth")).unwrap_err();
        assert!(matches!(err, RepositoryError::Internal(_)));
    }
}
