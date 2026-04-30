use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::StatusCode;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    RepositoryError,
    shared::pagination::{Page, Pagination},
    user::{User, UserCreate, UserRepository, UserRole},
};

use super::{
    client::KeycloakClient,
    mapping::{KcCredential, KcRoleRepresentation, KcUser},
};

const ATTR_USER_ROLES: &str = "user_roles";

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
            let body = resp.text().await.unwrap_or_default();
            return Err(RepositoryError::Internal(format!(
                "service-account token denied ({status}): {body}"
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
    async fn create(&self, entity: UserCreate) -> Result<User, RepositoryError> {
        let token = self.service_account_token().await?;

        let mut attributes: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(phone) = entity.phone_number.as_deref() {
            attributes.insert("phone_number".into(), vec![phone.into()]);
        }
        if let Some(emp) = entity.employee_id.as_deref() {
            attributes.insert("employee_id".into(), vec![emp.into()]);
        }
        if let Some(avatar) = entity.avatar_url.as_ref() {
            attributes.insert("avatar_url".into(), vec![avatar.to_string()]);
        }
        if !entity.roles.is_empty() {
            attributes.insert(ATTR_USER_ROLES.into(), vec![entity.roles.join(",")]);
        }

        let body = json!({
            "username": entity.username,
            "firstName": entity.first_name,
            "lastName": entity.last_name,
            "email": entity.email,
            "enabled": true,
            "emailVerified": false,
            "attributes": attributes,
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
                    entity.username
                )));
            }
            status => {
                let body = create_resp.text().await.unwrap_or_default();
                return Err(RepositoryError::Internal(format!(
                    "create user failed ({status}): {body}"
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
            let body = pwd_resp.text().await.unwrap_or_default();
            return Err(RepositoryError::Internal(format!(
                "reset-password failed ({status}): {body}"
            )));
        }

        if !entity.roles.is_empty() {
            self.assign_realm_roles(&token, &user_id, &entity.roles)
                .await?;
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
            let body = read_resp.text().await.unwrap_or_default();
            return Err(RepositoryError::Internal(format!(
                "read-back failed ({status}): {body}"
            )));
        }
        let kc_user: KcUser = read_resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("read-back parse: {e}")))?;
        kc_user.try_into_domain()
    }

    async fn all(&self, pagination: Pagination) -> Result<Page<User>, RepositoryError> {
        self.list_users(&[], pagination).await
    }

    async fn by_role(
        &self,
        role: UserRole,
        pagination: Pagination,
    ) -> Result<Page<User>, RepositoryError> {
        // Roles live in the `user_roles` custom attribute (not realm roles), so
        // we filter via the `q` param which Keycloak matches against attributes.
        let q = format!("user_roles:{}", role.as_str());
        self.list_users(&[("q", q.as_str())], pagination).await
    }

    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, RepositoryError> {
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
                    out.push(kc.try_into_domain()?);
                }
                StatusCode::NOT_FOUND => continue,
                status => {
                    let body = resp.text().await.unwrap_or_default();
                    return Err(RepositoryError::Internal(format!(
                        "by_ids failed ({status}): {body}"
                    )));
                }
            }
        }
        Ok(out)
    }

    async fn revoke_session(&self, refresh_token: &str) -> Result<(), RepositoryError> {
        // Keycloak's `/logout` (18+) terminates the session given the refresh token + owning client_id.
        let mut form: Vec<(&str, &str)> = vec![
            ("client_id", self.client.frontend_client_id.as_str()),
            ("refresh_token", refresh_token),
        ];
        if let Some(secret) = self.client.frontend_client_secret() {
            form.push(("client_secret", secret));
        }

        let resp = self
            .client
            .http()
            .post(self.client.logout_url())
            .form(&form)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("logout transport: {e}")))?;

        match resp.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED => {
                // Token already invalid — treat as success.
                Ok(())
            }
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(RepositoryError::Internal(format!(
                    "logout failed ({status}): {body}"
                )))
            }
        }
    }
}

impl KeycloakUserRepository {
    async fn list_users(
        &self,
        extra_query: &[(&str, &str)],
        pagination: Pagination,
    ) -> Result<Page<User>, RepositoryError> {
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
            let body = resp.text().await.unwrap_or_default();
            return Err(RepositoryError::Internal(format!(
                "list users failed ({status}): {body}"
            )));
        }
        let kc_users: Vec<KcUser> = resp
            .json()
            .await
            .map_err(|e| RepositoryError::Internal(format!("list users parse: {e}")))?;
        let items = kc_users
            .into_iter()
            .map(KcUser::try_into_domain)
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

    async fn assign_realm_roles(
        &self,
        token: &str,
        user_id: &str,
        role_names: &[String],
    ) -> Result<(), RepositoryError> {
        // Resolve role IDs by name.
        let mut wanted_roles: Vec<KcRole> = Vec::with_capacity(role_names.len());
        for name in role_names {
            let mut url = self.client.admin_realm_roles_url();
            url.path_segments_mut()
                .map_err(|_| RepositoryError::Internal("invalid roles url".into()))?
                .push(name);
            let resp = self
                .client
                .http()
                .get(url)
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| RepositoryError::Internal(format!("role lookup transport: {e}")))?;
            if !resp.status().is_success() {
                tracing::warn!(role = %name, status = %resp.status(), "role lookup failed; skipping");
                continue;
            }
            let role: KcRole = resp
                .json()
                .await
                .map_err(|e| RepositoryError::Internal(format!("role lookup parse: {e}")))?;
            wanted_roles.push(role);
        }

        if wanted_roles.is_empty() {
            return Ok(());
        }

        let payload: Vec<KcRoleRepresentation> = wanted_roles
            .iter()
            .map(|r| KcRoleRepresentation {
                id: r.id.as_str(),
                name: r.name.as_str(),
            })
            .collect();

        let resp = self
            .client
            .http()
            .post(self.client.admin_user_realm_role_mappings_url(user_id))
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("role assign transport: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RepositoryError::Internal(format!(
                "role assign failed ({status}): {body}"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct KcRole {
    pub id: String,
    pub name: String,
}
