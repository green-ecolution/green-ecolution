use domain::{Id, RepositoryError, plugin::Plugin};
use irrigation_domain::{
    plugin_refs::{PluginTreeRefs, TreeRef, TreeRefPage},
    tree::Tree,
};
use sqlx::PgPool;

pub struct PgPluginTreeRefs {
    pool: PgPool,
}

impl PgPluginTreeRefs {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PluginTreeRefs for PgPluginTreeRefs {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn tree_ref(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
    ) -> Result<Option<TreeRef>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT external_id, tree_id FROM plugin_tree_refs
               WHERE plugin_id = $1 AND external_id = $2"#,
            plugin.value(),
            external_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TreeRef {
            external_id: r.external_id,
            tree_id: Id::new(r.tree_id),
        }))
    }

    // `next_cursor` is set whenever the page came back full: a further page
    // *may* exist. If it happens not to, the caller sees one empty extra
    // page rather than the adapter risking an under-count on a page that
    // ends exactly at `limit`.
    #[tracing::instrument(level = "trace", skip_all)]
    async fn tree_refs(
        &self,
        plugin: Id<Plugin>,
        after: Option<&str>,
        limit: u32,
    ) -> Result<TreeRefPage, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT external_id, tree_id FROM plugin_tree_refs
               WHERE plugin_id = $1 AND ($2::text IS NULL OR external_id > $2)
               ORDER BY external_id ASC
               LIMIT $3"#,
            plugin.value(),
            after,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let next_cursor = (rows.len() == limit as usize)
            .then(|| rows.last().map(|r| r.external_id.clone()))
            .flatten();

        let items = rows
            .into_iter()
            .map(|row| TreeRef {
                external_id: row.external_id,
                tree_id: Id::new(row.tree_id),
            })
            .collect();

        Ok(TreeRefPage { items, next_cursor })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn link_tree(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
        tree: Id<Tree>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO plugin_tree_refs (plugin_id, external_id, tree_id)
               VALUES ($1, $2, $3)
               ON CONFLICT (plugin_id, external_id) DO UPDATE SET tree_id = EXCLUDED.tree_id"#,
            plugin.value(),
            external_id,
            tree.value()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn unlink_tree(
        &self,
        plugin: Id<Plugin>,
        external_id: &str,
    ) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"DELETE FROM plugin_tree_refs WHERE plugin_id = $1 AND external_id = $2"#,
            plugin.value(),
            external_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
