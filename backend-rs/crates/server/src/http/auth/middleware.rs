use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde_json::Value;
use uuid::Uuid;

use crate::{configuration::AuthSettings, infra::keycloak::JwksProvider, service::AuthError};
use domain::{auth::AuthUser, user::UserRole};

#[derive(Clone)]
pub struct AuthLayer {
    pub jwks: Arc<JwksProvider>,
    pub validation: Arc<Validation>,
    pub enabled: bool,
}

impl AuthLayer {
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
}

pub async fn auth_middleware(
    State(state): State<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    if !state.enabled {
        // Must stay in sync with `auth_service::dummy_token` claims so the demo identity is consistent.
        request.extensions_mut().insert(AuthUser {
            id: Uuid::nil(),
            username: Some("ttester".into()),
            email: Some("toni.tester@green-ecolution.de".into()),
            roles: vec![
                UserRole::Tbz,
                UserRole::GreenEcolution,
                UserRole::SmarteGrenzregion,
            ],
            raw_claims: Value::Null,
        });
        return Ok(next.run(request).await);
    }

    let auth_user = verify_bearer(&state, request.headers()).await?;
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

async fn verify_bearer(state: &AuthLayer, headers: &HeaderMap) -> Result<AuthUser, AuthError> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingToken)?;

    let header = decode_header(bearer)
        .map_err(|e| AuthError::InvalidToken(format!("malformed jwt header: {e}")))?;
    let kid = header
        .kid
        .ok_or_else(|| AuthError::InvalidToken("jwt missing kid header".into()))?;

    let key: DecodingKey = state.jwks.key_for_kid(&kid).await?;

    let token_data = decode::<Value>(bearer, &key, &state.validation).map_err(|e| {
        use jsonwebtoken::errors::ErrorKind;
        match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken(e.to_string()),
        }
    })?;

    claims_to_auth_user(token_data.claims)
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

    let roles = extract_roles(&claims);

    Ok(AuthUser {
        id,
        username,
        email,
        roles,
        raw_claims: claims,
    })
}

// Roles can live in `realm_access.roles`, `resource_access.<client>.roles`, or a custom
// `user_roles` claim depending on the Keycloak client mapper config — collect from all three.
fn extract_roles(claims: &Value) -> Vec<UserRole> {
    use std::str::FromStr;

    let mut out: Vec<UserRole> = Vec::new();
    let mut push_role = |s: &str| {
        if let Ok(role) = UserRole::from_str(s)
            && !out.contains(&role)
        {
            out.push(role);
        }
    };

    if let Some(arr) = claims
        .get("realm_access")
        .and_then(|v| v.get("roles"))
        .and_then(Value::as_array)
    {
        for v in arr {
            if let Some(s) = v.as_str() {
                push_role(s);
            }
        }
    }

    if let Some(obj) = claims.get("resource_access").and_then(Value::as_object) {
        for client in obj.values() {
            if let Some(arr) = client.get("roles").and_then(Value::as_array) {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        push_role(s);
                    }
                }
            }
        }
    }

    if let Some(arr) = claims.get("user_roles").and_then(Value::as_array) {
        for v in arr {
            if let Some(s) = v.as_str() {
                push_role(s);
            }
        }
    } else if let Some(s) = claims.get("user_roles").and_then(Value::as_str) {
        for piece in s.split(',') {
            push_role(piece.trim());
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_roles_from_realm_access() {
        let claims = json!({
            "sub": "11111111-2222-3333-4444-555555555555",
            "realm_access": { "roles": ["tbz", "offline_access"] }
        });
        let roles = extract_roles(&claims);
        assert_eq!(roles, vec![UserRole::Tbz]);
    }

    #[test]
    fn extracts_roles_from_user_roles_array() {
        let claims = json!({
            "sub": "11111111-2222-3333-4444-555555555555",
            "user_roles": ["green-ecolution", "tbz"]
        });
        let roles = extract_roles(&claims);
        assert!(roles.contains(&UserRole::GreenEcolution));
        assert!(roles.contains(&UserRole::Tbz));
    }

    #[test]
    fn extracts_roles_from_user_roles_csv_string() {
        let claims = json!({
            "user_roles": "tbz,smarte-grenzregion"
        });
        let roles = extract_roles(&claims);
        assert!(roles.contains(&UserRole::Tbz));
        assert!(roles.contains(&UserRole::SmarteGrenzregion));
    }
}
