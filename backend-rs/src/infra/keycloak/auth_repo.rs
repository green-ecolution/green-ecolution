use std::sync::Arc;

use chrono::{Duration, Utc};
use serde::Deserialize;
use url::Url;

use crate::domain::{
    RepositoryError,
    auth::{AuthRepository, ClientToken, IntrospectToken},
};

use super::client::KeycloakClient;

pub struct KeycloakAuthRepository {
    client: Arc<KeycloakClient>,
}

impl KeycloakAuthRepository {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self { client }
    }

    async fn post_token_form<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        form: &[(&str, &str)],
    ) -> Result<T, RepositoryError> {
        let response = self
            .client
            .http()
            .post(url)
            .form(form)
            .send()
            .await
            .map_err(map_transport)?;

        let status = response.status();
        let bytes = response.bytes().await.map_err(map_transport)?;

        if !status.is_success() {
            let body = std::str::from_utf8(&bytes).unwrap_or("<non-utf8>");
            return Err(map_oauth_error(status, body));
        }

        serde_json::from_slice::<T>(&bytes)
            .map_err(|e| RepositoryError::Internal(format!("failed to decode token response: {e}")))
    }
}

#[async_trait::async_trait]
impl AuthRepository for KeycloakAuthRepository {
    async fn introspect_token(&self, token: &str) -> Result<IntrospectToken, RepositoryError> {
        let response: IntrospectResponse = self
            .post_token_form(
                self.client.introspect_url(),
                &[
                    ("token", token),
                    ("client_id", &self.client.backend_client_id),
                    ("client_secret", self.client.backend_client_secret()),
                ],
            )
            .await?;

        Ok(IntrospectToken {
            exp: response.exp,
            active: Some(response.active),
            auth_time: response.auth_time,
            kind: response.token_type,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<ClientToken, RepositoryError> {
        let form: Vec<(&str, &str)> = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client.frontend_client_id),
        ];

        let resp: TokenResponse = self.post_token_form(self.client.token_url(), &form).await?;
        Ok(resp.into())
    }

    async fn access_token_from_client_code(
        &self,
        code: &str,
        redirect_url: &Url,
    ) -> Result<ClientToken, RepositoryError> {
        let redirect = redirect_url.to_string();
        let form: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect),
            ("client_id", &self.client.frontend_client_id),
        ];

        let resp: TokenResponse = self.post_token_form(self.client.token_url(), &form).await?;
        Ok(resp.into())
    }

    async fn access_token_from_client_credentials(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<ClientToken, RepositoryError> {
        let form: Vec<(&str, &str)> = vec![
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let resp: TokenResponse = self.post_token_form(self.client.token_url(), &form).await?;
        Ok(resp.into())
    }
}

fn map_transport(err: reqwest::Error) -> RepositoryError {
    if err.is_timeout() || err.is_connect() {
        RepositoryError::Internal(format!("idp unreachable: {err}"))
    } else {
        RepositoryError::Internal(format!("idp transport error: {err}"))
    }
}

fn map_oauth_error(status: reqwest::StatusCode, body: &str) -> RepositoryError {
    match status.as_u16() {
        400 | 401 => RepositoryError::ConstraintViolation(format!(
            "idp rejected token request ({status}): {body}"
        )),
        404 => RepositoryError::NotFound,
        _ => RepositoryError::Internal(format!("idp error ({status}): {body}")),
    }
}

#[derive(Debug, Deserialize)]
struct IntrospectResponse {
    active: bool,
    #[serde(default)]
    exp: Option<u32>,
    #[serde(default)]
    auth_time: Option<u32>,
    #[serde(default)]
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    id_token: Option<String>,
    expires_in: u32,
    #[serde(default)]
    refresh_expires_in: Option<u32>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default = "default_token_type")]
    token_type: String,
    #[serde(default, rename = "not-before-policy")]
    not_before_policy: Option<u32>,
    #[serde(default)]
    session_state: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

fn default_token_type() -> String {
    "Bearer".into()
}

impl From<TokenResponse> for ClientToken {
    fn from(value: TokenResponse) -> Self {
        let expiry = Utc::now() + Duration::seconds(value.expires_in.into());
        ClientToken {
            access_token: value.access_token,
            id_token: value.id_token.unwrap_or_default(),
            expiry,
            expires_in: value.expires_in,
            refresh_expires_in: value.refresh_expires_in.unwrap_or(0),
            refresh_token: value.refresh_token.unwrap_or_default(),
            token_type: value.token_type,
            not_before_policy: value.not_before_policy.unwrap_or(0),
            session_state: value.session_state.unwrap_or_default(),
            scope: value.scope.unwrap_or_default(),
        }
    }
}
