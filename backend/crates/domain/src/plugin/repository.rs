use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    Id, RepositoryError,
    plugin::{Plugin, PluginDraft, PluginKeyHash, PluginSlug, PluginView},
};

#[async_trait]
pub trait PluginReader: Send + Sync {
    async fn by_id(&self, id: Id<Plugin>) -> Result<Plugin, RepositoryError>;
    async fn by_slug(&self, slug: &PluginSlug) -> Result<Plugin, RepositoryError>;
    async fn all(&self) -> Result<Vec<PluginView>, RepositoryError>;
    async fn view_by_slug(&self, slug: &PluginSlug) -> Result<PluginView, RepositoryError>;
    async fn last_seen_at(&self, id: Id<Plugin>) -> Result<Option<DateTime<Utc>>, RepositoryError>;
}

#[async_trait]
pub trait PluginWriter: Send + Sync {
    /// Takes the id from the caller rather than minting its own (unlike every
    /// other `save_new` in this codebase): the API key returned alongside the
    /// plugin embeds this id, so the caller must know it before the row
    /// exists.
    async fn save_new(
        &self,
        id: Id<Plugin>,
        draft: PluginDraft,
        key_hash: Option<PluginKeyHash>,
    ) -> Result<Plugin, RepositoryError>;
    async fn save(&self, plugin: &Plugin) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Plugin>) -> Result<(), RepositoryError>;
    async fn touch_last_seen(&self, id: Id<Plugin>) -> Result<(), RepositoryError>;
}
