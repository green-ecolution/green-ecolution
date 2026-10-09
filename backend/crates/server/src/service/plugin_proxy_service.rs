use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    plugin::{Plugin, PluginFrontend, PluginReader, PluginSlug, ServiceEndpoint},
};

use super::{
    AuthError, ServiceError, authorization::AuthorizationService, plugin_proxy_policy::ProxyPolicy,
    plugin_view_ticket_service::PluginViewTicketService,
};

/// Same reasoning as `ViewTicketFactory`: a CSPRNG and a digest are adapter
/// concerns, kept out of a layer that must stay portable.
pub trait ProxySessionFactory: Send + Sync {
    /// Returns the plaintext token and the hash to persist.
    fn generate(&self) -> (String, String);
    /// `None` for anything that is not shaped like a token this factory issues.
    fn hash(&self, token: &str) -> Option<String>;
}

pub struct NewProxySession {
    pub token_hash: String,
    pub plugin_id: Id<Plugin>,
    pub user_id: Uuid,
    pub user_display_name: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProxySession {
    pub plugin_id: Id<Plugin>,
    pub user_id: Uuid,
    pub user_display_name: String,
}

#[async_trait]
pub trait ProxySessionStore: Send + Sync {
    async fn insert(&self, session: NewProxySession) -> Result<(), RepositoryError>;
    /// `None` for an unknown or expired token.
    async fn find(&self, token_hash: &str) -> Result<Option<ProxySession>, RepositoryError>;
    async fn purge_expired(&self, plugin: Id<Plugin>) -> Result<(), RepositoryError>;
}

/// What the proxy may forward a request with: where to, and on whose behalf.
#[derive(Debug, Clone)]
pub struct ProxyGrant {
    pub endpoint: ServiceEndpoint,
    pub user_id: Uuid,
    pub user_display_name: String,
}

/// Turns a view ticket into a browser session on the plugin's own host and
/// re-checks that session on every request, so a revoked role or a disabled
/// plugin takes effect immediately instead of when the session expires.
pub struct PluginProxyService {
    plugins: Arc<dyn PluginReader>,
    tickets: Arc<PluginViewTicketService>,
    authorization: Arc<AuthorizationService>,
    sessions: Arc<dyn ProxySessionStore>,
    tokens: Arc<dyn ProxySessionFactory>,
    policy: ProxyPolicy,
    session_ttl: chrono::Duration,
}

impl PluginProxyService {
    pub fn new(
        plugins: Arc<dyn PluginReader>,
        tickets: Arc<PluginViewTicketService>,
        authorization: Arc<AuthorizationService>,
        sessions: Arc<dyn ProxySessionStore>,
        tokens: Arc<dyn ProxySessionFactory>,
        policy: ProxyPolicy,
        session_ttl: chrono::Duration,
    ) -> Self {
        Self {
            plugins,
            tickets,
            authorization,
            sessions,
            tokens,
            policy,
            session_ttl,
        }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plugin.slug = %slug.as_str()))]
    pub async fn open_session(
        &self,
        slug: &PluginSlug,
        ticket: &str,
    ) -> Result<String, ServiceError> {
        let plugin = self.plugins.by_slug(slug).await?;
        // Before the redeem, so a failing purge leaves the one-time ticket usable.
        self.sessions.purge_expired(plugin.id).await?;
        let holder = self.tickets.redeem(&plugin, ticket).await?;

        let (token, token_hash) = self.tokens.generate();
        self.sessions
            .insert(NewProxySession {
                token_hash,
                plugin_id: plugin.id,
                user_id: holder.user_id,
                user_display_name: holder.user_display_name,
                expires_at: Utc::now() + self.session_ttl,
            })
            .await?;
        Ok(token)
    }

    /// The session is checked before the plugin is loaded, so a visitor
    /// without one cannot tell an unknown slug from a known one.
    #[tracing::instrument(level = "debug", skip_all, fields(plugin.slug = %slug.as_str()))]
    pub async fn authorize(
        &self,
        slug: &PluginSlug,
        token: Option<&str>,
    ) -> Result<ProxyGrant, ServiceError> {
        let session = match token.and_then(|t| self.tokens.hash(t)) {
            Some(hash) => self.sessions.find(&hash).await?,
            None => None,
        }
        .ok_or(AuthError::PluginProxySessionInvalid)?;

        let plugin = self.plugins.by_slug(slug).await?;
        if session.plugin_id != plugin.id {
            return Err(AuthError::PluginProxySessionInvalid.into());
        }
        if !plugin.enabled() {
            return Err(AuthError::PluginDisabled.into());
        }
        let PluginFrontend::Proxied(endpoint) = plugin.frontend() else {
            return Err(ServiceError::PluginProxyTargetNotAllowed);
        };
        if !self.policy.allows(endpoint) {
            return Err(ServiceError::PluginProxyTargetNotAllowed);
        }
        self.authorization
            .require_plugin_view(
                session.user_id,
                plugin.required_permissions(),
                plugin.organization_id(),
            )
            .await?;

        Ok(ProxyGrant {
            endpoint: endpoint.clone(),
            user_id: session.user_id,
            user_display_name: session.user_display_name,
        })
    }
}
