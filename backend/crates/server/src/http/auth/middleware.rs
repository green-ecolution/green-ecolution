use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, header},
    middleware::Next,
    response::Response,
};
use domain::{Id, organization::Organization};
use uuid::Uuid;

use crate::{
    configuration::AuthSettings, http::auth::validator::TokenValidator,
    infra::keycloak::JwksProvider, service::AuthError,
};

/// The organization a request is narrowed to. Only parsed here; whether the
/// user may pick it is decided by `AuthorizationService::context_for`.
pub const ORGANIZATION_SCOPE_HEADER: &str = "x-organization-scope";

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
    let mut user = if state.validator.is_enforced() {
        let token = extract_bearer(request.headers()).ok_or(AuthError::MissingToken)?;
        state.validator.validate(&token).await?
    } else {
        TokenValidator::anonymous_user()
    };
    user.organization_scope = organization_scope(request.headers())?;
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

fn organization_scope(headers: &HeaderMap) -> Result<Option<Id<Organization>>, AuthError> {
    headers
        .get(ORGANIZATION_SCOPE_HEADER)
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|s| Uuid::parse_str(s).ok())
                .map(Id::new)
                .ok_or(AuthError::InvalidOrganizationScope)
        })
        .transpose()
}
