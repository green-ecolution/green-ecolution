use secrecy::{ExposeSecret, SecretString};

use domain::data_export::ExportAuth;

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
            let token: TokenResponse = response
                .json()
                .await
                .map_err(|e| AuthFailure::Malformed(e.without_url().to_string()))?;
            Ok(Authorization::Bearer(SecretString::from(
                token.access_token,
            )))
        }
    }
}
