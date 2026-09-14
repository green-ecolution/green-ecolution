use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    Id, RepositoryError,
    plugin::{Plugin, PluginDraft, PluginKeyHash, PluginSlug, PluginView},
    tree::Tree,
};

/// One external-key to tree mapping owned by a plugin.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeRef {
    pub external_id: String,
    pub tree_id: Id<Tree>,
}

/// Keyset page. Deliberately not `shared::pagination::Page`, which carries a
/// `total` and therefore an extra COUNT: the adapter walks this list to the
/// end anyway, and a cursor stays stable while imports run concurrently.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeRefPage {
    pub items: Vec<TreeRef>,
    pub next_cursor: Option<String>,
}

#[async_trait]
pub trait PluginReader: Send + Sync {
    async fn by_id(&self, id: Id<Plugin>) -> Result<Plugin, RepositoryError>;
    async fn by_slug(&self, slug: &PluginSlug) -> Result<Plugin, RepositoryError>;
    async fn all(&self) -> Result<Vec<PluginView>, RepositoryError>;
    async fn view_by_slug(&self, slug: &PluginSlug) -> Result<PluginView, RepositoryError>;
    async fn last_seen_at(&self, id: Id<Plugin>) -> Result<Option<DateTime<Utc>>, RepositoryError>;

    /// Resolves the tree a plugin's external key points at, if any.
    async fn tree_ref(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
    ) -> Result<Option<TreeRef>, RepositoryError>;

    /// Keyset page over the plugin's own refs, ordered by `external_id`.
    /// `after` is the last `external_id` of the previous page.
    async fn tree_refs(
        &self,
        plugin: Id<Plugin>,
        after: Option<&str>,
        limit: u32,
    ) -> Result<TreeRefPage, RepositoryError>;
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
    async fn link_tree(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
        tree: Id<Tree>,
    ) -> Result<(), RepositoryError>;
    async fn unlink_tree(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
    ) -> Result<(), RepositoryError>;
}
