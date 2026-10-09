use async_trait::async_trait;
use sqlx::PgPool;

use domain::{Id, RepositoryError, plugin::Plugin};

use crate::service::plugin_proxy_service::{NewProxySession, ProxySession, ProxySessionStore};

pub struct PgProxySessionStore {
    pool: PgPool,
}

impl PgProxySessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProxySessionStore for PgProxySessionStore {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn insert(&self, session: NewProxySession) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO plugin_proxy_sessions
                   (token_hash, plugin_id, user_id, user_display_name, expires_at)
               VALUES ($1, $2, $3, $4, $5)"#,
            session.token_hash,
            session.plugin_id.value(),
            session.user_id,
            session.user_display_name,
            session.expires_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn find(&self, token_hash: &str) -> Result<Option<ProxySession>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT plugin_id, user_id, user_display_name
                 FROM plugin_proxy_sessions
                WHERE token_hash = $1 AND expires_at > now()"#,
            token_hash,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| ProxySession {
            plugin_id: Id::new(r.plugin_id),
            user_id: r.user_id,
            user_display_name: r.user_display_name,
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn purge_expired(&self, plugin: Id<Plugin>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "DELETE FROM plugin_proxy_sessions WHERE plugin_id = $1 AND expires_at <= now()",
            plugin.value(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
