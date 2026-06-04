//! Auth feature bootstrap. Constructs the Keycloak-backed auth stack
//! and starts the JWKS refresher when `enabled = true`.

use std::sync::Arc;

use crate::{
    configuration::AuthSettings,
    http::auth::AuthLayer,
    service::user_service::UserService,
};

use super::{JwksProvider, KeycloakClient, KeycloakUserRepository};

/// Composed auth dependencies returned by [`build`].
///
/// The HTTP layer takes [`AuthLayer`]; [`AppState`](crate::http::AppState)
/// takes the service. `_jwks` is held by `Application` purely for
/// ownership: dropping it would stop the background refresh loop.
pub struct AuthStack {
    pub user_service: Arc<UserService>,
    pub auth_layer: AuthLayer,
    pub jwks: Arc<JwksProvider>,
}

/// Builds the auth stack from settings. When `enabled = false` the same
/// objects are returned (unified shape for callers) but the JWKS refresher
/// does not start and downstream consumers fall back to bypass behaviour.
pub async fn build(settings: &AuthSettings) -> Result<AuthStack, std::io::Error> {
    let kc_client = Arc::new(
        KeycloakClient::new(settings)
            .map_err(|e| std::io::Error::other(format!("keycloak client init: {e}")))?,
    );
    let jwks = Arc::new(JwksProvider::new(&kc_client, settings));
    if settings.enabled {
        // Soft-fail: dev environments without a running Keycloak can still
        // boot; the background refresher will pick up keys once it comes
        // online.
        if let Err(err) = jwks.refresh_now().await {
            tracing::warn!(error = %err, "initial JWKS refresh failed; will retry in background");
        }
        jwks.spawn_background_refresh();
    } else {
        tracing::info!(
            "auth disabled due to config (auth.enabled = false); JWT validation bypassed"
        );
    }

    let user_repo = Arc::new(KeycloakUserRepository::new(kc_client));
    let user_service = Arc::new(UserService::new(user_repo, settings.enabled));
    let auth_layer = AuthLayer::new(jwks.clone(), settings);

    Ok(AuthStack {
        user_service,
        auth_layer,
        jwks,
    })
}
