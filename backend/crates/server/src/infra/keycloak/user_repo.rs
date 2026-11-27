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
            StatusCode::BAD_REQUEST => {
                return Err(RepositoryError::ConstraintViolation(
                    "identity provider rejected the input".into(),
                ));
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

        match self.finish_account(&token, &user_id, &entity).await {
            Ok(identity) => Ok(identity),
            Err(error) => {
                if let Err(cleanup) = self.delete_account(&token, &user_id).await {
                    tracing::error!(
                        user_id = %user_id,
                        error = %cleanup,
                        "could not remove the account of a failed registration"
                    );
                }
                Err(error)
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let token = self.service_account_token().await?;
        self.delete_account(&token, &id.to_string()).await
    }

    async fn all(&self, pagination: Pagination) -> Result<Page<UserIdentity>, RepositoryError> {
        self.list_users(&[], pagination).await
    }

    async fn search(
        &self,
        query: &str,
        pagination: Pagination,
    ) -> Result<Page<UserIdentity>, RepositoryError> {
        // Keycloak's `search` param is prefix-based by default ("foo" only matches
        // names/emails starting with "foo"); wrapping in `*` forces infix matching so
        // this actually delivers the containment search the rest of the system promises.
        let wrapped = format!("*{query}*");
        self.list_users(&[("search", &wrapped)], pagination).await
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
    async fn finish_account(
        &self,
        token: &str,
        user_id: &str,
        entity: &UserIdentityCreate,
    ) -> Result<UserIdentity, RepositoryError> {
        let pwd_resp = self
            .client
            .http()
            .put(self.client.admin_user_reset_password_url(user_id))
            .bearer_auth(token)
            .json(&KcCredential {
                kind: "password",
                value: entity.password.expose_secret(),
                temporary: true,
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
            .get(self.client.admin_user_url(user_id))
            .bearer_auth(token)
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

    async fn delete_account(&self, token: &str, user_id: &str) -> Result<(), RepositoryError> {
        let resp = self
            .client
            .http()
            .delete(self.client.admin_user_url(user_id))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("delete user transport: {e}")))?;
        match resp.status() {
            status if status.is_success() => Ok(()),
            StatusCode::NOT_FOUND => Ok(()),
            status => Err(RepositoryError::Internal(format!(
                "delete user failed ({status})"
            ))),
        }
    }

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

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_partial_json, method, path},
    };

    use domain::{shared::email::Email, user::Username};

    use super::*;
    use crate::configuration::AuthSettings;

    const REALM: &str = "green-ecolution";

    fn repo(server: &MockServer) -> KeycloakUserRepository {
        let settings = AuthSettings {
            enabled: true,
            issuer_url: format!("{}/realms/{REALM}", server.uri()),
            frontend_client_id: "frontend".into(),
            backend_client_id: "backend".into(),
            backend_client_secret: SecretString::from("backend-secret".to_string()),
            jwks_refresh_interval_secs: 3600,
            jwks_refresh_timeout_secs: 5,
            default_redirect_url: "http://127.0.0.1/cb".into(),
            expected_audience: None,
        };
        KeycloakUserRepository::new(Arc::new(KeycloakClient::new(&settings).unwrap()))
    }

    fn entity() -> UserIdentityCreate {
        UserIdentityCreate {
            username: Username::reconstitute("ttester".into()),
            first_name: "Toni".into(),
            last_name: "Tester".into(),
            email: Email::reconstitute("toni@example.com".into()),
            password: SecretString::from("Abcde-fghjk-mnpqr-stuvw".to_string()),
        }
    }

    fn users_path() -> String {
        format!("/admin/realms/{REALM}/users")
    }

    fn user_path(id: Uuid) -> String {
        format!("{}/{id}", users_path())
    }

    async fn mock_token(server: &MockServer) {
        Mock::given(method("POST"))
            .and(path(format!(
                "/realms/{REALM}/protocol/openid-connect/token"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "service-token",
                "expires_in": 300,
            })))
            .mount(server)
            .await;
    }

    async fn mock_create(server: &MockServer, id: Uuid) {
        Mock::given(method("POST"))
            .and(path(users_path()))
            .respond_with(ResponseTemplate::new(201).insert_header(
                "Location",
                format!("{}{}", server.uri(), user_path(id)).as_str(),
            ))
            .mount(server)
            .await;
    }

    async fn mock_password(server: &MockServer, id: Uuid, status: u16) {
        Mock::given(method("PUT"))
            .and(path(format!("{}/reset-password", user_path(id))))
            .respond_with(ResponseTemplate::new(status))
            .mount(server)
            .await;
    }

    async fn mock_read(server: &MockServer, id: Uuid, status: u16) {
        let response = if status == 200 {
            ResponseTemplate::new(200).set_body_json(json!({
                "id": id.to_string(),
                "username": "ttester",
                "firstName": "Toni",
                "lastName": "Tester",
                "email": "toni@example.com",
                "emailVerified": false,
            }))
        } else {
            ResponseTemplate::new(status)
        };
        Mock::given(method("GET"))
            .and(path(user_path(id)))
            .respond_with(response)
            .mount(server)
            .await;
    }

    async fn mock_delete(server: &MockServer, id: Uuid, status: u16, expected: u64) {
        Mock::given(method("DELETE"))
            .and(path(user_path(id)))
            .respond_with(ResponseTemplate::new(status))
            .expect(expected)
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn create_sets_the_password_as_temporary() {
        let server = MockServer::start().await;
        let id = Uuid::now_v7();
        mock_token(&server).await;
        mock_create(&server, id).await;
        Mock::given(method("PUT"))
            .and(path(format!("{}/reset-password", user_path(id))))
            .and(body_partial_json(
                json!({"type": "password", "temporary": true}),
            ))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        mock_read(&server, id, 200).await;
        mock_delete(&server, id, 204, 0).await;

        let identity = repo(&server).create(entity()).await.unwrap();

        assert_eq!(identity.id, id);
    }

    #[tokio::test]
    async fn create_removes_the_account_when_the_password_cannot_be_set() {
        let server = MockServer::start().await;
        let id = Uuid::now_v7();
        mock_token(&server).await;
        mock_create(&server, id).await;
        mock_password(&server, id, 500).await;
        mock_delete(&server, id, 204, 1).await;

        let result = repo(&server).create(entity()).await;

        assert!(matches!(result, Err(RepositoryError::Internal(_))));
    }

    #[tokio::test]
    async fn create_removes_the_account_when_it_cannot_be_read_back() {
        let server = MockServer::start().await;
        let id = Uuid::now_v7();
        mock_token(&server).await;
        mock_create(&server, id).await;
        mock_password(&server, id, 204).await;
        mock_read(&server, id, 500).await;
        mock_delete(&server, id, 204, 1).await;

        let result = repo(&server).create(entity()).await;

        assert!(matches!(result, Err(RepositoryError::Internal(_))));
    }

    #[tokio::test]
    async fn create_keeps_the_original_error_when_the_cleanup_fails_too() {
        let server = MockServer::start().await;
        let id = Uuid::now_v7();
        mock_token(&server).await;
        mock_create(&server, id).await;
        mock_password(&server, id, 500).await;
        mock_delete(&server, id, 500, 1).await;

        let result = repo(&server).create(entity()).await;

        match result {
            Err(RepositoryError::Internal(message)) => {
                assert!(message.contains("reset-password"), "{message}");
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_taken_username_or_email_is_reported_as_already_existing() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path(users_path()))
            .respond_with(ResponseTemplate::new(409))
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let result = repo(&server).create(entity()).await;

        assert!(matches!(result, Err(RepositoryError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn input_the_identity_provider_refuses_is_a_constraint_violation() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path(users_path()))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(json!({"errorMessage": "error-invalid-length"})),
            )
            .mount(&server)
            .await;
        Mock::given(method("DELETE"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let result = repo(&server).create(entity()).await;

        match result {
            Err(RepositoryError::ConstraintViolation(message)) => {
                assert_eq!(message, "identity provider rejected the input");
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_treats_a_missing_account_as_done() {
        for (status, ok) in [(204, true), (404, true), (500, false)] {
            let server = MockServer::start().await;
            let id = Uuid::now_v7();
            mock_token(&server).await;
            mock_delete(&server, id, status, 1).await;

            let result = repo(&server).delete(id).await;

            if ok {
                assert!(result.is_ok(), "status {status}");
            } else {
                assert!(
                    matches!(result, Err(RepositoryError::Internal(_))),
                    "status {status}"
                );
            }
        }
    }
}
