use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::StatusCode;
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use uuid::Uuid;

use domain::{
    RepositoryError,
    shared::pagination::{Page, Pagination},
    user::{UserIdentity, UserIdentityCreate, UserRepository},
};

use super::{
    client::KeycloakClient,
    mapping::{KcCredential, KcUser},
};

pub struct KeycloakUserRepository {
    client: Arc<KeycloakClient>,
    service_token: Mutex<Option<CachedServiceToken>>,
}

struct CachedServiceToken {
    access_token: String,
    expires_at: Instant,
}

impl KeycloakUserRepository {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self {
            client,
            service_token: Mutex::new(None),
        }
    }

    async fn service_account_token(&self) -> Result<String, RepositoryError> {
        let mut guard = self.service_token.lock().await;
        if let Some(cached) = guard.as_ref()
            && cached.expires_at > Instant::now() + Duration::from_secs(5)
        {
            return Ok(cached.access_token.clone());
        }

        let resp = self
            .client
            .http()
            .post(self.client.token_url())
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client.backend_client_id.as_str()),
                ("client_secret", self.client.backend_client_secret()),
            ])
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("service-account token failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(RepositoryError::Internal(format!(
                "service-account token denied ({status})"
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: u64,
        }

        let token: TokenResponse = resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("invalid token response: {e}")))?;

        let expires_at = Instant::now() + Duration::from_secs(token.expires_in.saturating_sub(30));
        *guard = Some(CachedServiceToken {
            access_token: token.access_token.clone(),
            expires_at,
        });
        Ok(token.access_token)
    }
}

#[async_trait::async_trait]
impl UserRepository for KeycloakUserRepository {
    async fn create(&self, entity: UserIdentityCreate) -> Result<UserIdentity, RepositoryError> {
        let token = self.service_account_token().await?;

        let body = json!({
            "username": entity.username.as_str(),
            "firstName": entity.first_name,
            "lastName": entity.last_name,
            "email": entity.email.as_str(),
            "enabled": true,
            "emailVerified": false,
        });

        let create_resp = self
            .client
            .http()
            .post(self.client.admin_users_url())
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("create user transport: {e}")))?;

        match create_resp.status() {
            StatusCode::CREATED => {}
            StatusCode::CONFLICT => {
                return Err(RepositoryError::AlreadyExists(format!(
                    "username/email already exists: {}",
                    entity.username.as_str()
                )));
            }
            status => {
                return Err(RepositoryError::Internal(format!(
                    "create user failed ({status})"
                )));
            }
        }

        let user_id = create_resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|loc| loc.rsplit('/').next().map(str::to_string))
            .ok_or_else(|| {
                RepositoryError::Internal("create user response missing Location header".into())
            })?;

        let pwd_resp = self
            .client
            .http()
            .put(self.client.admin_user_reset_password_url(&user_id))
            .bearer_auth(&token)
            .json(&KcCredential {
                kind: "password",
                value: entity.password.expose_secret(),
                temporary: false,
            })
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("reset-password transport: {e}")))?;
        if !pwd_resp.status().is_success() {
            let status = pwd_resp.status();
            return Err(RepositoryError::Internal(format!(
                "reset-password failed ({status})"
            )));
        }

        let read_resp = self
            .client
            .http()
            .get(self.client.admin_user_url(&user_id))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("read-back transport: {e}")))?;
        if !read_resp.status().is_success() {
            let status = read_resp.status();
            return Err(RepositoryError::Internal(format!(
                "read-back failed ({status})"
            )));
        }
        let kc_user: KcUser = read_resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("read-back parse: {e}")))?;
        kc_user.try_into_identity()
    }

    async fn all(&self, pagination: Pagination) -> Result<Page<UserIdentity>, RepositoryError> {
        self.list_users(&[], pagination).await
    }

    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserIdentity>, RepositoryError> {
        let token = self.service_account_token().await?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            let resp = self
                .client
                .http()
                .get(self.client.admin_user_url(&id.to_string()))
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| RepositoryError::Internal(format!("by_ids transport: {e}")))?;
            match resp.status() {
                StatusCode::OK => {
                    let kc: KcUser = resp
                        .json()
                        .await
                        .map_err(|e| RepositoryError::Internal(format!("by_ids parse: {e}")))?;
                    out.push(kc.try_into_identity()?);
                }
                StatusCode::NOT_FOUND => continue,
                status => {
                    return Err(RepositoryError::Internal(format!(
                        "by_ids failed ({status})"
                    )));
                }
            }
        }
        Ok(out)
    }
}

impl KeycloakUserRepository {
    async fn list_users(
        &self,
        extra_query: &[(&str, &str)],
        pagination: Pagination,
    ) -> Result<Page<UserIdentity>, RepositoryError> {
        let token = self.service_account_token().await?;
        let total = self.count_users(&token, extra_query).await?;

        let mut url = self.client.admin_users_url();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("first", &pagination.offset().to_string());
            q.append_pair("max", &pagination.limit().to_string());
            for (k, v) in extra_query {
                q.append_pair(k, v);
            }
        }

        let resp = self
            .client
            .http()
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("list users transport: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(RepositoryError::Internal(format!(
                "list users failed ({status})"
            )));
        }
        let kc_users: Vec<KcUser> = resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("list users parse: {e}")))?;
        let items = kc_users
            .into_iter()
            .map(KcUser::try_into_identity)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page { items, total })
    }

    async fn count_users(
        &self,
        token: &str,
        extra_query: &[(&str, &str)],
    ) -> Result<u64, RepositoryError> {
        let mut url = self.client.admin_users_url();
        url.path_segments_mut()
            .map_err(|_| RepositoryError::Internal("invalid users url".into()))?
            .push("count");
        if !extra_query.is_empty() {
            let mut q = url.query_pairs_mut();
            for (k, v) in extra_query {
                q.append_pair(k, v);
            }
        }
        let resp = self
            .client
            .http()
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("count users transport: {e}")))?;
        if !resp.status().is_success() {
            return Ok(0);
        }
        let count: u64 = resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("count users parse: {e}")))?;
        Ok(count)
    }
}
