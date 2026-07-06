use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, header},
    middleware::Next,
    response::Response,
};

use crate::{
    configuration::AuthSettings, http::auth::validator::TokenValidator,
    infra::keycloak::JwksProvider, service::AuthError,
};

#[derive(Clone)]
pub struct AuthLayer {
    pub validator: Arc<TokenValidator>,
}

impl AuthLayer {
    pub fn new(jwks: Arc<JwksProvider>, settings: &AuthSettings) -> Self {
        Self {
            validator: Arc::new(TokenValidator::new(jwks, settings)),
        }
    }
}

pub async fn auth_middleware(
    State(state): State<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    if !state.validator.is_enforced() {
        request
            .extensions_mut()
            .insert(TokenValidator::anonymous_user());
        return Ok(next.run(request).await);
    }

    let token = extract_bearer(request.headers()).ok_or(AuthError::MissingToken)?;
    let user = state.validator.validate(&token).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_string)
}
