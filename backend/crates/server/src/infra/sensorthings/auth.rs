use secrecy::{ExposeSecret, SecretString};

use domain::data_export::{ExportAuth, SinkError};

use super::body::read_capped;

/// A token answer is a few hundred bytes; anything near this is not one.
pub(crate) const MAX_TOKEN_BYTES: usize = 64 * 1024;

pub(crate) enum Authorization {
    None,
    Basic {
        username: String,
        password: SecretString,
    },
    Bearer(SecretString),
}

impl Authorization {
    pub(crate) fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self {
            Self::None => request,
            Self::Basic { username, password } => {
                request.basic_auth(username, Some(password.expose_secret()))
            }
            Self::Bearer(token) => request.bearer_auth(token.expose_secret()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum AuthFailure {
    Unreachable(String),
    Rejected { status: u16 },
    Malformed(String),
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub(crate) async fn authorize(
    client: &reqwest::Client,
    auth: &ExportAuth,
) -> Result<Authorization, AuthFailure> {
    match auth {
        ExportAuth::None => Ok(Authorization::None),
        ExportAuth::Basic { username, password } => Ok(Authorization::Basic {
            username: username.as_str().to_string(),
            password: SecretString::from(password.expose().to_string()),
        }),
        ExportAuth::OAuth2ClientCredentials {
            token_url,
            client_id,
            client_secret,
            scope,
        } => {
            let mut form = vec![
                ("grant_type", "client_credentials"),
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.expose()),
            ];
            if let Some(scope) = scope {
                form.push(("scope", scope.as_str()));
            }
            let response = client
                .post(token_url.as_url().clone())
                .form(&form)
                .send()
                .await
                .map_err(|e| AuthFailure::Unreachable(e.without_url().to_string()))?;
            if !response.status().is_success() {
                return Err(AuthFailure::Rejected {
                    status: response.status().as_u16(),
                });
            }
            let body = read_capped(response, MAX_TOKEN_BYTES)
                .await
                .map_err(|e| match e {
                    SinkError::Unreachable(detail) => AuthFailure::Unreachable(detail),
                    other => AuthFailure::Malformed(other.to_string()),
                })?;
            let token: TokenResponse =
                serde_json::from_slice(&body).map_err(|e| AuthFailure::Malformed(e.to_string()))?;
            Ok(Authorization::Bearer(SecretString::from(
                token.access_token,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::data_export::{ExportPrincipal, ExportSecret, ExportUrl, TOKEN_URL_FIELD};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::infra::sensorthings::http_client;

    fn oauth2(server: &MockServer) -> ExportAuth {
        ExportAuth::OAuth2ClientCredentials {
            token_url: ExportUrl::new(TOKEN_URL_FIELD, &format!("{}/token", server.uri())).unwrap(),
            client_id: ExportPrincipal::new("ge").unwrap(),
            client_secret: ExportSecret::new("f", SecretString::from("s".to_string())).unwrap(),
            scope: None,
        }
    }

    #[tokio::test]
    async fn an_oversized_token_answer_is_malformed() {
        let server = MockServer::start().await;
        let padding = "x".repeat(MAX_TOKEN_BYTES);
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!(r#"{{"access_token":"tok","padding":"{padding}"}}"#)),
            )
            .mount(&server)
            .await;

        let result = authorize(&http_client(), &oauth2(&server)).await;

        assert!(matches!(result, Err(AuthFailure::Malformed(_))));
    }

    #[tokio::test]
    async fn a_token_answer_below_the_cap_is_used() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"access_token":"tok"}"#))
            .mount(&server)
            .await;

        let result = authorize(&http_client(), &oauth2(&server)).await;

        assert!(matches!(result, Ok(Authorization::Bearer(t)) if t.expose_secret() == "tok"));
    }
}
