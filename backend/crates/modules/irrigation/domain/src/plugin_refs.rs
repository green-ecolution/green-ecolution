//! The mapping from a plugin's own keys to the trees it imported.

use async_trait::async_trait;
use domain::{Id, RepositoryError, plugin::Plugin};

use crate::tree::Tree;

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
pub trait PluginTreeRefs: Send + Sync {
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
