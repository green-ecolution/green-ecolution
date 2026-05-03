use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};
use url::Url;

use crate::{configuration::AuthSettings, service::AuthError};

use super::client::KeycloakClient;

// `refresh_lock` serializes refreshes so concurrent cache misses don't
// stampede the IdP — only one fetch in flight, the rest re-check the cache.
pub struct JwksProvider {
    state: RwLock<JwksState>,
    refresh_lock: Mutex<()>,
    http: reqwest::Client,
    jwks_url: Url,
    refresh_interval: Duration,
    refresh_timeout: Duration,
}

struct JwksState {
    keys: HashMap<String, DecodingKey>,
    last_refresh: Option<Instant>,
}

impl JwksProvider {
    pub fn new(client: &KeycloakClient, settings: &AuthSettings) -> Self {
        Self {
            state: RwLock::new(JwksState {
                keys: HashMap::new(),
                last_refresh: None,
            }),
            refresh_lock: Mutex::new(()),
            http: client.http().clone(),
            jwks_url: client.jwks_url(),
            refresh_interval: Duration::from_secs(settings.jwks_refresh_interval_secs.max(1)),
            refresh_timeout: Duration::from_secs(settings.jwks_refresh_timeout_secs.max(1)),
        }
    }

    pub async fn key_for_kid(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        if let Some(key) = self.cached_key(kid).await {
            return Ok(key);
        }

        self.refresh_now().await?;

        self.cached_key(kid)
            .await
            .ok_or_else(|| AuthError::InvalidToken(format!("unknown signing kid: {kid}")))
    }

    pub async fn refresh_now(&self) -> Result<(), AuthError> {
        let _guard = self.refresh_lock.lock().await;

        // Bail if another waiter just refreshed; avoids redundant fetches under contention.
        {
            let state = self.state.read().await;
            if let Some(last) = state.last_refresh
                && last.elapsed() < Duration::from_secs(5)
                && !state.keys.is_empty()
            {
                return Ok(());
            }
        }

        let response = self
            .http
            .get(self.jwks_url.clone())
            .timeout(self.refresh_timeout)
            .send()
            .await
            .map_err(|e| AuthError::IdpUnavailable(format!("jwks fetch failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AuthError::IdpUnavailable(format!(
                "jwks fetch returned status {}",
                response.status()
            )));
        }

        let document: JwksDocument = response
            .json()
            .await
            .map_err(|e| AuthError::IdpUnavailable(format!("jwks parse failed: {e}")))?;

        let mut keys = HashMap::with_capacity(document.keys.len());
        for jwk in document.keys {
            if jwk.use_.as_deref() == Some("enc") {
                continue;
            }
            if jwk.kty != "RSA" {
                continue;
            }
            let (Some(n), Some(e)) = (jwk.n.as_deref(), jwk.e.as_deref()) else {
                continue;
            };
            match DecodingKey::from_rsa_components(n, e) {
                Ok(key) => {
                    keys.insert(jwk.kid, key);
                }
                Err(err) => {
                    tracing::warn!(
                        kid = %jwk.kid,
                        error = %err,
                        "failed to build decoding key from jwk"
                    );
                }
            }
        }

        if keys.is_empty() {
            return Err(AuthError::IdpUnavailable(
                "jwks document contained no usable RSA signing keys".into(),
            ));
        }

        let mut state = self.state.write().await;
        state.keys = keys;
        state.last_refresh = Some(Instant::now());
        tracing::info!(count = state.keys.len(), "loaded JWKS keys");
        Ok(())
    }

    async fn cached_key(&self, kid: &str) -> Option<DecodingKey> {
        self.state.read().await.keys.get(kid).cloned()
    }

    pub fn spawn_background_refresh(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval = self.refresh_interval;
        let weak = Arc::downgrade(self);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await; // tokio::time::interval fires immediately on first tick.
            loop {
                ticker.tick().await;
                let Some(provider) = weak.upgrade() else {
                    return;
                };
                if let Err(err) = provider.refresh_now().await {
                    tracing::warn!(error = %err, "scheduled JWKS refresh failed");
                }
            }
        })
    }
}

#[derive(Deserialize)]
struct JwksDocument {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    #[serde(default)]
    n: Option<String>,
    #[serde(default)]
    e: Option<String>,
    #[serde(rename = "use", default)]
    use_: Option<String>,
}
