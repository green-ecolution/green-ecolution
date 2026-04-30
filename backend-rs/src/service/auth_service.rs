use std::sync::Arc;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::json;
use url::Url;

use crate::{
    configuration::AuthSettings,
    domain::auth::{AuthRepository, ClientToken, IntrospectToken, LoginResponse},
};

use super::ServiceError;

const DUMMY_TOKEN_LIFETIME_SECS: u32 = 365 * 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct AuthServiceConfig {
    pub enabled: bool,
    pub frontend_client_id: String,
    pub default_redirect_url: Url,
    pub auth_url: Url,
}

impl AuthServiceConfig {
    pub fn from_settings(settings: &AuthSettings) -> Result<Self, ServiceError> {
        let issuer_url = Url::parse(&settings.issuer_url)
            .map_err(|e| ServiceError::InvalidInput(format!("invalid issuer_url: {e}")))?;
        let mut auth_url = issuer_url.clone();
        let path = format!("{}/protocol/openid-connect/auth", issuer_url.path());
        auth_url.set_path(&path);
        let default_redirect_url = Url::parse(&settings.default_redirect_url).map_err(|e| {
            ServiceError::InvalidInput(format!("invalid default_redirect_url: {e}"))
        })?;
        Ok(Self {
            enabled: settings.enabled,
            frontend_client_id: settings.frontend_client_id.clone(),
            default_redirect_url,
            auth_url,
        })
    }
}

pub struct AuthService {
    auth_repo: Arc<dyn AuthRepository>,
    config: AuthServiceConfig,
}

impl AuthService {
    pub fn new(auth_repo: Arc<dyn AuthRepository>, config: AuthServiceConfig) -> Self {
        Self { auth_repo, config }
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn login_url(&self, redirect_url: Option<Url>) -> LoginResponse {
        let redirect = redirect_url.unwrap_or_else(|| self.config.default_redirect_url.clone());
        if !self.config.enabled {
            // Skip the IdP roundtrip: the SPA callback URL with code=demo
            // already appended is enough for the dummy flow.
            let mut url = redirect;
            url.query_pairs_mut().append_pair("code", "demo");
            return LoginResponse { login_url: url };
        }
        let mut url = self.config.auth_url.clone();
        url.query_pairs_mut()
            .clear()
            .append_pair("client_id", &self.config.frontend_client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid profile email")
            .append_pair("redirect_uri", redirect.as_str());
        LoginResponse { login_url: url }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_url: Url,
    ) -> Result<ClientToken, ServiceError> {
        if !self.config.enabled {
            return Ok(dummy_token());
        }
        Ok(self
            .auth_repo
            .access_token_from_client_code(code, &redirect_url)
            .await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh(&self, refresh_token: &str) -> Result<ClientToken, ServiceError> {
        if !self.config.enabled {
            return Ok(dummy_token());
        }
        Ok(self.auth_repo.refresh_token(refresh_token).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn introspect(&self, token: &str) -> Result<IntrospectToken, ServiceError> {
        if !self.config.enabled {
            return Ok(IntrospectToken {
                exp: None,
                active: Some(true),
                auth_time: None,
                kind: Some("Bearer".into()),
            });
        }
        Ok(self.auth_repo.introspect_token(token).await?)
    }
}

// JWT-shaped (`header.payload.signature`) so the frontend's decodeJWT picks
// up the claims from the middle segment. Header/signature are garbage — the
// middleware skips signature verification when `auth.enabled = false`.
fn dummy_token() -> ClientToken {
    let claims = json!({
        "email": "toni.tester@green-ecolution.de",
        "preferred_username": "ttester",
        "given_name": "Toni",
        "family_name": "Tester",
        "driving_licenses": ["B", "BE", "C", "CE"],
        "user_roles": ["green-ecolution"],
        "status": "available",
    });
    let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap_or_default());
    let access_token = format!("lsidu.{payload_b64}.oicsxfusd");
    let refresh_token = format!("sinxoled.{payload_b64}.sldkfjalf");
    ClientToken {
        access_token,
        id_token: String::new(),
        expiry: Utc::now() + Duration::seconds(DUMMY_TOKEN_LIFETIME_SECS.into()),
        expires_in: DUMMY_TOKEN_LIFETIME_SECS,
        refresh_expires_in: DUMMY_TOKEN_LIFETIME_SECS,
        refresh_token,
        token_type: "Bearer".into(),
        not_before_policy: 0,
        session_state: String::new(),
        scope: String::new(),
    }
}
