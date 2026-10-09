use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain::{Id, RepositoryError, plugin::Plugin};

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
