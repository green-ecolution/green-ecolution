use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde_json::Value;
use uuid::Uuid;

use crate::{configuration::AuthSettings, infra::keycloak::JwksProvider, service::AuthError};
use domain::auth::AuthUser;

pub struct TokenValidator {
    jwks: Arc<JwksProvider>,
    validation: Arc<Validation>,
    enabled: bool,
}

impl TokenValidator {
    pub fn new(jwks: Arc<JwksProvider>, settings: &AuthSettings) -> Self {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&settings.issuer_url]);
        if let Some(aud) = &settings.expected_audience {
            validation.set_audience(&[aud]);
        } else {
            // Keycloak's default `aud` is "account"; don't enforce unless explicitly configured.
            validation.validate_aud = false;
        }
        validation.validate_exp = true;
        validation.leeway = 30;

        Self {
            jwks,
            validation: Arc::new(validation),
            enabled: settings.enabled,
        }
    }

    pub fn is_enforced(&self) -> bool {
        self.enabled
    }

    pub fn anonymous_user() -> AuthUser {
        // Must match the anonymous demo user injected by auth_middleware when auth.enabled = false.
        AuthUser {
            id: Uuid::nil(),
            username: Some("ttester".into()),
            email: Some("toni.tester@green-ecolution.de".into()),
            raw_claims: Value::Null,
        }
    }

    pub async fn validate(&self, token: &str) -> Result<AuthUser, AuthError> {
        if !self.enabled {
            return Ok(Self::anonymous_user());
        }

        let header = decode_header(token)
            .map_err(|e| AuthError::InvalidToken(format!("malformed jwt header: {e}")))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("jwt missing kid header".into()))?;

        let key: DecodingKey = self.jwks.key_for_kid(&kid).await?;

        let token_data = decode::<Value>(token, &key, &self.validation).map_err(|e| {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            }
        })?;

        claims_to_auth_user(token_data.claims)
    }
}

fn claims_to_auth_user(claims: Value) -> Result<AuthUser, AuthError> {
    let sub = claims
        .get("sub")
        .and_then(Value::as_str)
        .ok_or_else(|| AuthError::InvalidToken("missing sub claim".into()))?;
    let id = Uuid::parse_str(sub)
        .map_err(|e| AuthError::InvalidToken(format!("sub is not a uuid: {e}")))?;

    let username = claims
        .get("preferred_username")
        .and_then(Value::as_str)
        .map(str::to_string);
    let email = claims
        .get("email")
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(AuthUser {
        id,
        username,
        email,
        raw_claims: claims,
    })
}
