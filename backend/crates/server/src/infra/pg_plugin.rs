use std::sync::Arc;

use sqlx::PgPool;

use domain::plugin::{
    Plugin, PluginDraft, PluginFrontend, PluginKeyHash, PluginReader, PluginSlug, PluginSnapshot,
    PluginView, PluginWriter,
};
use domain::{Id, RepositoryError, authorization::ResourceCatalog};

pub struct PgPluginRepository {
    pool: PgPool,
    catalog: Arc<ResourceCatalog>,
}

impl PgPluginRepository {
    pub fn new(pool: PgPool, catalog: Arc<ResourceCatalog>) -> Self {
        Self { pool, catalog }
    }
}

fn frontend_parts(frontend: &PluginFrontend) -> (&'static str, Option<String>) {
    match frontend {
        PluginFrontend::None => ("none", None),
        PluginFrontend::External(url) => ("external", Some(url.to_string())),
        PluginFrontend::Proxied(ep) => ("proxied", Some(format!("{}:{}", ep.host(), ep.port()))),
    }
}

#[async_trait::async_trait]
impl PluginReader for PgPluginRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Plugin>) -> Result<Plugin, RepositoryError> {
        sqlx::query_as!(
            PluginSnapshot,
            r#"SELECT id, slug, name, description, frontend_mode, frontend_target,
                      organization_id, permissions, required_permissions, device_capabilities,
                      enabled, key_hash, last_seen_at
               FROM plugins WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)
        .map(|snap| Plugin::reconstitute(snap, &self.catalog))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_slug(&self, slug: &PluginSlug) -> Result<Plugin, RepositoryError> {
        sqlx::query_as!(
            PluginSnapshot,
            r#"SELECT id, slug, name, description, frontend_mode, frontend_target,
                      organization_id, permissions, required_permissions, device_capabilities,
                      enabled, key_hash, last_seen_at
               FROM plugins WHERE slug = $1"#,
            slug.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)
        .map(|snap| Plugin::reconstitute(snap, &self.catalog))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn all(&self) -> Result<Vec<PluginView>, RepositoryError> {
        let snapshots = sqlx::query_as!(
            PluginSnapshot,
            r#"SELECT id, slug, name, description, frontend_mode, frontend_target,
                      organization_id, permissions, required_permissions, device_capabilities,
                      enabled, key_hash, last_seen_at
               FROM plugins ORDER BY name ASC, id ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snapshots
            .into_iter()
            .map(|snap| {
                let last_seen_at = snap.last_seen_at;
                let plugin = Plugin::reconstitute(snap, &self.catalog);
                PluginView::from_aggregate(&plugin, last_seen_at)
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_slug(&self, slug: &PluginSlug) -> Result<PluginView, RepositoryError> {
        let snap = sqlx::query_as!(
            PluginSnapshot,
            r#"SELECT id, slug, name, description, frontend_mode, frontend_target,
                      organization_id, permissions, required_permissions, device_capabilities,
                      enabled, key_hash, last_seen_at
               FROM plugins WHERE slug = $1"#,
            slug.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let last_seen_at = snap.last_seen_at;
        let plugin = Plugin::reconstitute(snap, &self.catalog);
        Ok(PluginView::from_aggregate(&plugin, last_seen_at))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn last_seen_at(
        &self,
        id: Id<Plugin>,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT last_seen_at FROM plugins WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(row.last_seen_at)
    }
}

#[async_trait::async_trait]
impl PluginWriter for PgPluginRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(
        &self,
        id: Id<Plugin>,
        draft: PluginDraft,
        key_hash: Option<PluginKeyHash>,
    ) -> Result<Plugin, RepositoryError> {
        let (frontend_mode, frontend_target) = frontend_parts(&draft.frontend);
        let permissions: Vec<String> = draft.permissions.iter().map(|p| p.to_string()).collect();
        let device_capabilities: Vec<String> = draft
            .device_capabilities
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        let required_permissions: Vec<String> = draft
            .required_permissions
            .iter()
            .map(|p| p.to_string())
            .collect();

        sqlx::query!(
            r#"INSERT INTO plugins (id, slug, name, description, frontend_mode, frontend_target,
                                     organization_id, permissions, required_permissions,
                                     device_capabilities, enabled, key_hash)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, FALSE, $11)"#,
            id.value(),
            draft.slug.as_str(),
            draft.name.as_str(),
            draft.description.as_deref(),
            frontend_mode,
            frontend_target.as_deref(),
            draft.organization_id.value(),
            &permissions,
            &required_permissions,
            &device_capabilities,
            key_hash.as_ref().map(|k| k.as_str()),
        )
        .execute(&self.pool)
        .await?;

        Ok(Plugin::reconstitute(
            PluginSnapshot {
                id: id.value(),
                slug: draft.slug.as_str().to_string(),
                name: draft.name.as_str().to_string(),
                description: draft.description,
                frontend_mode: frontend_mode.to_string(),
                frontend_target,
                organization_id: draft.organization_id.value(),
                permissions,
                required_permissions,
                device_capabilities,
                enabled: false,
                key_hash: key_hash.map(|k| k.as_str().to_string()),
                last_seen_at: None,
            },
            &self.catalog,
        ))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, plugin: &Plugin) -> Result<(), RepositoryError> {
        let (frontend_mode, frontend_target) = frontend_parts(plugin.frontend());
        let permissions: Vec<String> = plugin.permissions().iter().map(|p| p.to_string()).collect();
        let device_capabilities: Vec<String> = plugin
            .device_capabilities()
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        let required_permissions: Vec<String> = plugin
            .required_permissions()
            .iter()
            .map(|p| p.to_string())
            .collect();

        let result = sqlx::query!(
            r#"UPDATE plugins
               SET name = $2, description = $3, frontend_mode = $4, frontend_target = $5,
                   organization_id = $6, permissions = $7, required_permissions = $8,
                   enabled = $9, key_hash = $10, device_capabilities = $11
               WHERE id = $1"#,
            plugin.id.value(),
            plugin.name.as_str(),
            plugin.description.as_deref(),
            frontend_mode,
            frontend_target.as_deref(),
            plugin.organization_id().value(),
            &permissions,
            &required_permissions,
            plugin.enabled(),
            plugin.key_hash().map(|k| k.as_str()),
            &device_capabilities,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Plugin>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM plugins WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn touch_last_seen(&self, id: Id<Plugin>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE plugins SET last_seen_at = now() WHERE id = $1"#,
            id.value()
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
