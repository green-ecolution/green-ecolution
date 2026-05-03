use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{domain::auth::AuthUser, service::AuthError};

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AuthError::MissingToken)
    }
}
