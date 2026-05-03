use std::ops::Deref;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::service::AuthError;
use domain::auth::AuthUser;

/// Local newtype that implements [`FromRequestParts`] for [`AuthUser`].
///
/// Required because the orphan rule prevents implementing a foreign trait
/// (`FromRequestParts`) for a foreign type (`AuthUser` from the `domain` crate)
/// directly in the server crate.
#[derive(Debug, Clone)]
pub struct AuthUserExtractor(pub AuthUser);

impl Deref for AuthUserExtractor {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for AuthUserExtractor
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(AuthUserExtractor)
            .ok_or(AuthError::MissingToken)
    }
}
