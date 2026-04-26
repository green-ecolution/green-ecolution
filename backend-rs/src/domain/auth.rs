use chrono::{DateTime, Utc};
use url::Url;

use crate::domain::RepositoryError;

#[derive(Debug, Clone)]
pub struct IntrospectToken {
    pub exp: Option<u32>,
    pub active: Option<bool>,
    pub auth_time: Option<u32>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClientToken {
    pub access_token: String,
    pub id_token: String,
    pub expiry: DateTime<Utc>,
    pub expires_in: u32,
    pub refresh_expires_in: u32,
    pub refresh_token: String,
    pub token_type: String,
    pub not_before_policy: u32,
    pub session_state: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub redirect_url: Url,
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub login_url: Url,
}

#[derive(Debug, Clone)]
pub struct LoginCallback {
    pub code: String,
    pub redirect_url: Url,
}

#[derive(Debug, Clone)]
pub struct Logout {
    pub refresh_token: String,
}

#[async_trait::async_trait]
pub trait AuthRepository: Send + Sync {
    async fn introspect_token(&self, token: &str) -> Result<IntrospectToken, RepositoryError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<ClientToken, RepositoryError>;
    async fn access_token_from_client_code(
        &self,
        token: &str,
        redirect_url: &Url,
    ) -> Result<ClientToken, RepositoryError>;
    async fn access_token_from_client_credentials(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<ClientToken, RepositoryError>;
}
