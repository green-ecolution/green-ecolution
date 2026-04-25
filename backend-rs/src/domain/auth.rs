use chrono::{DateTime, Utc};
use url::Url;

use crate::domain::RepositoryError;

#[derive(Debug, Clone)]
pub struct IntrospectToken {
    exp: Option<u32>,
    active: Option<bool>,
    auth_time: Option<u32>,
    kind: Option<String>,
}

impl IntrospectToken {
    pub fn new(
        exp: Option<u32>,
        active: Option<bool>,
        auth_time: Option<u32>,
        kind: Option<String>,
    ) -> Self {
        Self {
            exp,
            active,
            auth_time,
            kind,
        }
    }

    pub fn exp(&self) -> Option<u32> {
        self.exp
    }
    pub fn active(&self) -> Option<bool> {
        self.active
    }
    pub fn auth_time(&self) -> Option<u32> {
        self.auth_time
    }
    pub fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct ClientToken {
    access_token: String,
    id_token: String,
    expiry: DateTime<Utc>,
    expires_in: u32,
    refresh_expires_in: u32,
    refresh_token: String,
    token_type: String,
    not_before_policy: u32,
    session_state: String,
    scope: String,
}

impl ClientToken {
    pub fn new(
        access_token: String,
        id_token: String,
        expiry: DateTime<Utc>,
        expires_in: u32,
        refresh_expires_in: u32,
        refresh_token: String,
        token_type: String,
        not_before_policy: u32,
        session_state: String,
        scope: String,
    ) -> Self {
        Self {
            access_token,
            id_token,
            expiry,
            expires_in,
            refresh_expires_in,
            refresh_token,
            token_type,
            not_before_policy,
            session_state,
            scope,
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    pub fn id_token(&self) -> &str {
        &self.id_token
    }
    pub fn expiry(&self) -> DateTime<Utc> {
        self.expiry
    }
    pub fn expires_in(&self) -> u32 {
        self.expires_in
    }
    pub fn refresh_expires_in(&self) -> u32 {
        self.refresh_expires_in
    }
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    pub fn token_type(&self) -> &str {
        &self.token_type
    }
    pub fn not_before_policy(&self) -> u32 {
        self.not_before_policy
    }
    pub fn session_state(&self) -> &str {
        &self.session_state
    }
    pub fn scope(&self) -> &str {
        &self.scope
    }
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    redirect_url: Url,
}

impl LoginRequest {
    pub fn new(redirect_url: Url) -> Self {
        Self { redirect_url }
    }

    pub fn redirect_url(&self) -> &Url {
        &self.redirect_url
    }
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    login_url: Url,
}

impl LoginResponse {
    pub fn new(login_url: Url) -> Self {
        Self { login_url }
    }

    pub fn login_url(&self) -> &Url {
        &self.login_url
    }
}

#[derive(Debug, Clone)]
pub struct LoginCallback {
    code: String,
    redirect_url: Url,
}

impl LoginCallback {
    pub fn new(code: String, redirect_url: Url) -> Self {
        Self { code, redirect_url }
    }

    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn redirect_url(&self) -> &Url {
        &self.redirect_url
    }
}

#[derive(Debug, Clone)]
pub struct Logout {
    refresh_token: String,
}

impl Logout {
    pub fn new(refresh_token: String) -> Self {
        Self { refresh_token }
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
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
