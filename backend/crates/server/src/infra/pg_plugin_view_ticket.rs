use async_trait::async_trait;
use sqlx::PgPool;

use domain::{Id, RepositoryError, plugin::Plugin};

use crate::service::plugin_view_ticket_service::{NewViewTicket, RedeemedTicket, ViewTicketStore};

pub struct PgViewTicketStore {
    pool: PgPool,
}

impl PgViewTicketStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ViewTicketStore for PgViewTicketStore {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn insert(&self, ticket: NewViewTicket) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO plugin_view_tickets
                   (token_hash, plugin_id, user_id, user_display_name, expires_at)
               VALUES ($1, $2, $3, $4, $5)"#,
            ticket.token_hash,
            ticket.plugin_id.value(),
            ticket.user_id,
            ticket.user_display_name,
            ticket.expires_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn redeem(
        &self,
        plugin: Id<Plugin>,
        token_hash: &str,
    ) -> Result<Option<RedeemedTicket>, RepositoryError> {
        // A single conditional UPDATE keeps redemption atomic under concurrent requests.
        let row = sqlx::query!(
            r#"UPDATE plugin_view_tickets
                  SET redeemed_at = now()
                WHERE token_hash = $1
                  AND plugin_id = $2
                  AND redeemed_at IS NULL
                  AND expires_at > now()
            RETURNING user_id, user_display_name"#,
            token_hash,
            plugin.value(),
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| RedeemedTicket {
            user_id: r.user_id,
            user_display_name: r.user_display_name,
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn purge_expired(&self, plugin: Id<Plugin>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "DELETE FROM plugin_view_tickets WHERE plugin_id = $1 AND expires_at <= now()",
            plugin.value(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
