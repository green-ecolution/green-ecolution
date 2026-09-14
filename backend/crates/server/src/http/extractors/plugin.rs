use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use domain::{RepositoryError, plugin::Plugin};

use crate::{
    http::AppState,
    infra::plugin_key,
    service::{AuthError, Feature, ServiceError},
};

/// Authenticated plugin behind an API key. Separate from `AuthUserExtractor`
/// because a plugin is not a person and carries no Keycloak identity.
pub struct PluginPrincipal(pub Plugin);

impl FromRequestParts<Arc<AppState>> for PluginPrincipal {
    type Rejection = ServiceError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        if !state.feature_flags.plugins_enabled {
            return Err(ServiceError::FeatureDisabled {
                feature: Feature::Plugins,
            });
        }

        let raw = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(AuthError::MissingToken)?;

        let (id, secret) = plugin_key::parse_key(raw).ok_or(AuthError::PluginKeyInvalid)?;
        // Only `NotFound` means "this key names no plugin". Every other
        // repository failure (a dead pool, `Internal`, `DataIntegrity`, ...) is
        // a real infrastructure problem and must surface as its own 5xx and get
        // logged, not disappear behind an unauthenticated-looking 401.
        let plugin = match state.plugin_reader.by_id(domain::Id::new(id)).await {
            Ok(plugin) => plugin,
            Err(RepositoryError::NotFound) => return Err(AuthError::PluginKeyInvalid.into()),
            Err(e) => return Err(e.into()),
        };

        let stored = plugin.key_hash().ok_or(AuthError::PluginKeyInvalid)?;
        if !constant_time_eq(stored.as_str(), plugin_key::hash_secret(&secret).as_str()) {
            return Err(AuthError::PluginKeyInvalid.into());
        }
        if !plugin.enabled() {
            return Err(AuthError::PluginDisabled.into());
        }

        // Every authenticated request is a sign of life, which is why there is
        // no separate heartbeat endpoint. A failed write must not fail the
        // request itself.
        let _ = state.plugin_writer.touch_last_seen(plugin.id).await;

        Ok(Self(plugin))
    }
}

/// Runs in constant time so a timing side channel cannot leak how much of a
/// guessed key matched the stored hash.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn equal_strings_match() {
        assert!(constant_time_eq("abc123", "abc123"));
    }

    #[test]
    fn different_strings_do_not_match() {
        assert!(!constant_time_eq("abc123", "abc124"));
    }

    #[test]
    fn different_lengths_do_not_match() {
        assert!(!constant_time_eq("abc", "abc123"));
    }

    #[test]
    fn empty_never_matches_a_hash() {
        assert!(!constant_time_eq("", "abc123"));
    }
}
