use domain::data_export::{
    BASE_URL_FIELD, ConnectionOutcome, ConnectionReport, ConnectionTester, DATA_ARRAY_CONFORMANCE,
    ExportTargetConfig, ExportUrlPolicy, TOKEN_URL_FIELD,
};

use super::auth::{AuthFailure, authorize};
#[cfg(test)]
use super::http_client;

pub struct SensorThingsProbe {
    client: reqwest::Client,
    policy: ExportUrlPolicy,
}

impl SensorThingsProbe {
    pub fn new(client: reqwest::Client, policy: ExportUrlPolicy) -> Self {
        Self { client, policy }
    }
}

#[derive(serde::Deserialize)]
struct ServiceRoot {
    #[allow(dead_code)] // reason: only its presence marks a SensorThings root
    value: Vec<serde_json::Value>,
    #[serde(default, rename = "serverSettings")]
    server_settings: Option<ServerSettings>,
}

#[derive(serde::Deserialize)]
struct ServerSettings {
    #[serde(default)]
    conformance: Vec<String>,
}

#[async_trait::async_trait]
impl ConnectionTester for SensorThingsProbe {
    #[tracing::instrument(level = "info", skip_all, fields(target.host = config.base_url.as_url().host_str()))]
    async fn test(&self, config: &ExportTargetConfig) -> ConnectionReport {
        let allowed = self.policy.check(BASE_URL_FIELD, &config.base_url).is_ok()
            && config
                .auth
                .token_url()
                .is_none_or(|url| self.policy.check(TOKEN_URL_FIELD, url).is_ok());
        if !allowed {
            return ConnectionReport::failed(ConnectionOutcome::UrlNotAllowed, None, None);
        }

        let authorization = match authorize(&self.client, &config.auth).await {
            Ok(a) => a,
            Err(AuthFailure::Unreachable(detail)) => {
                return ConnectionReport::failed(
                    ConnectionOutcome::Unreachable,
                    None,
                    Some(detail),
                );
            }
            Err(AuthFailure::Rejected { status }) => {
                return ConnectionReport::failed(
                    ConnectionOutcome::Unauthorized,
                    Some(status),
                    None,
                );
            }
            Err(AuthFailure::Malformed(detail)) => {
                return ConnectionReport::failed(
                    ConnectionOutcome::Unauthorized,
                    None,
                    Some(detail),
                );
            }
        };

        let response = match authorization
            .apply(self.client.get(config.base_url.as_url().clone()))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return ConnectionReport::failed(
                    ConnectionOutcome::Unreachable,
                    None,
                    Some(e.without_url().to_string()),
                );
            }
        };
        let status = response.status().as_u16();
        if matches!(status, 401 | 403) {
            return ConnectionReport::failed(ConnectionOutcome::Unauthorized, Some(status), None);
        }
        if response.status().is_server_error() {
            return ConnectionReport::failed(ConnectionOutcome::Unreachable, Some(status), None);
        }
        if !response.status().is_success() {
            return ConnectionReport::failed(
                ConnectionOutcome::NotSensorThings,
                Some(status),
                None,
            );
        }
        match response.json::<ServiceRoot>().await {
            Ok(root) => ConnectionReport {
                outcome: ConnectionOutcome::Ok,
                http_status: Some(status),
                data_array_supported: root
                    .server_settings
                    .is_some_and(|s| s.conformance.iter().any(|c| c == DATA_ARRAY_CONFORMANCE)),
                detail: None,
            },
            Err(_) => {
                ConnectionReport::failed(ConnectionOutcome::NotSensorThings, Some(status), None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::data_export::{
        BASE_URL_FIELD, ExportAuth, ExportPrincipal, ExportSecret, ExportTargetConfig, ExportUrl,
        ExportUrlPolicy, TOKEN_URL_FIELD,
    };
    use secrecy::SecretString;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    fn policy() -> ExportUrlPolicy {
        ExportUrlPolicy::new(["127.0.0.1".to_string()], true)
    }

    fn config(server: &MockServer, auth: ExportAuth) -> ExportTargetConfig {
        ExportTargetConfig {
            enabled: true,
            base_url: ExportUrl::new(
                BASE_URL_FIELD,
                &format!("{}/FROST-Server/v1.1", server.uri()),
            )
            .unwrap(),
            auth,
        }
    }

    fn secret(v: &str) -> ExportSecret {
        ExportSecret::new("f", SecretString::from(v.to_string())).unwrap()
    }

    fn root_body(conformance: &[&str]) -> serde_json::Value {
        json!({
            "value": [{ "name": "Things", "url": "x" }],
            "serverSettings": { "conformance": conformance }
        })
    }

    async fn mount_root(server: &MockServer, status: u16, body: serde_json::Value) {
        Mock::given(method("GET"))
            .and(path("/FROST-Server/v1.1"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn a_sensorthings_root_with_data_array_is_ok() {
        let server = MockServer::start().await;
        mount_root(&server, 200, root_body(&[DATA_ARRAY_CONFORMANCE])).await;

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, ExportAuth::None))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Ok);
        assert!(report.data_array_supported);
    }

    #[tokio::test]
    async fn a_root_without_data_array_is_ok_but_says_so() {
        let server = MockServer::start().await;
        mount_root(&server, 200, root_body(&[])).await;

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, ExportAuth::None))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Ok);
        assert!(!report.data_array_supported);
    }

    #[tokio::test]
    async fn basic_auth_is_sent_and_a_401_is_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/FROST-Server/v1.1"))
            .and(header("authorization", "Basic Z2U6cHc="))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let auth = ExportAuth::Basic {
            username: ExportPrincipal::new("ge").unwrap(),
            password: secret("pw"),
        };

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, auth))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Unauthorized);
        assert_eq!(report.http_status, Some(401));
    }

    #[tokio::test]
    async fn oauth2_fetches_a_token_and_uses_it_as_bearer() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("grant_type=client_credentials"))
            .and(body_string_contains("client_id=ge"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "tok", "token_type": "Bearer", "expires_in": 300
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/FROST-Server/v1.1"))
            .and(header("authorization", "Bearer tok"))
            .respond_with(ResponseTemplate::new(200).set_body_json(root_body(&[])))
            .mount(&server)
            .await;
        let auth = ExportAuth::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, &format!("{}/token", server.uri())).unwrap(),
            client_id: ExportPrincipal::new("ge").unwrap(),
            client_secret: secret("s"),
            scope: None,
        };

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, auth))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Ok);
    }

    #[tokio::test]
    async fn a_rejected_token_request_is_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;
        let auth = ExportAuth::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, &format!("{}/token", server.uri())).unwrap(),
            client_id: ExportPrincipal::new("ge").unwrap(),
            client_secret: secret("s"),
            scope: None,
        };

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, auth))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Unauthorized);
        assert_eq!(report.http_status, Some(400));
    }

    #[tokio::test]
    async fn redirects_are_not_followed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/FROST-Server/v1.1"))
            .respond_with(
                ResponseTemplate::new(302).insert_header("location", "http://169.254.169.254/"),
            )
            .mount(&server)
            .await;

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, ExportAuth::None))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::NotSensorThings);
        assert_eq!(report.http_status, Some(302));
    }

    #[tokio::test]
    async fn a_server_error_on_the_root_is_unreachable() {
        let server = MockServer::start().await;
        mount_root(&server, 503, json!({})).await;

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, ExportAuth::None))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Unreachable);
        assert_eq!(report.http_status, Some(503));
    }

    #[tokio::test]
    async fn something_else_than_sensorthings_is_reported() {
        let server = MockServer::start().await;
        mount_root(&server, 200, json!({ "hello": "world" })).await;

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, ExportAuth::None))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::NotSensorThings);
    }

    #[tokio::test]
    async fn a_host_off_the_list_is_never_called() {
        let server = MockServer::start().await;
        let report = SensorThingsProbe::new(
            http_client(),
            ExportUrlPolicy::new(Vec::<String>::new(), true),
        )
        .test(&config(&server, ExportAuth::None))
        .await;

        assert_eq!(report.outcome, ConnectionOutcome::UrlNotAllowed);
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_token_url_off_the_list_is_never_called() {
        let server = MockServer::start().await;
        let auth = ExportAuth::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, "https://auth.evil.test/token").unwrap(),
            client_id: ExportPrincipal::new("ge").unwrap(),
            client_secret: secret("s"),
            scope: None,
        };

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&config(&server, auth))
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::UrlNotAllowed);
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn an_unreachable_server_is_reported() {
        // A dropped MockServer goes back to wiremock's pool and keeps listening.
        let closed_port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let cfg = ExportTargetConfig {
            enabled: true,
            base_url: ExportUrl::new(
                BASE_URL_FIELD,
                &format!("http://127.0.0.1:{closed_port}/FROST-Server/v1.1"),
            )
            .unwrap(),
            auth: ExportAuth::None,
        };

        let report = SensorThingsProbe::new(http_client(), policy())
            .test(&cfg)
            .await;

        assert_eq!(report.outcome, ConnectionOutcome::Unreachable);
    }
}
