use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use domain::{Id, RepositoryError, auth::AuthUser, plugin::Plugin};

use super::{AuthError, ServiceError};

const TTL_SECONDS: i64 = 120;

/// Mints view tickets and derives the hash a ticket is stored under. Same
/// reasoning as `PluginKeyFactory`: a CSPRNG and a digest are adapter
/// concerns, kept out of a layer that must stay portable.
pub trait ViewTicketFactory: Send + Sync {
    /// Returns the plaintext ticket and the hash to persist.
    fn generate(&self) -> (String, String);
    /// `None` for anything that is not shaped like a ticket this factory issues.
    fn hash(&self, ticket: &str) -> Option<String>;
}

pub struct NewViewTicket {
    pub token_hash: String,
    pub plugin_id: Id<Plugin>,
    pub user_id: Uuid,
    pub user_display_name: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedeemedTicket {
    pub user_id: Uuid,
    pub user_display_name: String,
}

#[async_trait]
pub trait ViewTicketStore: Send + Sync {
    async fn insert(&self, ticket: NewViewTicket) -> Result<(), RepositoryError>;
    /// Marks the ticket redeemed and returns its holder, or `None` when it is
    /// unknown, expired, already redeemed or bound to another plugin.
    async fn redeem(
        &self,
        plugin: Id<Plugin>,
        token_hash: &str,
    ) -> Result<Option<RedeemedTicket>, RepositoryError>;
    async fn purge_expired(&self, plugin: Id<Plugin>) -> Result<(), RepositoryError>;
}

/// Tells a plugin's backend who opened its view. A ticket grants nothing
/// inside Green Ecolution; it only names the person in front of the screen.
pub struct PluginViewTicketService {
    store: Arc<dyn ViewTicketStore>,
    tickets: Arc<dyn ViewTicketFactory>,
}

impl PluginViewTicketService {
    pub fn new(store: Arc<dyn ViewTicketStore>, tickets: Arc<dyn ViewTicketFactory>) -> Self {
        Self { store, tickets }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin))]
    pub async fn issue(&self, plugin: Id<Plugin>, user: &AuthUser) -> Result<String, ServiceError> {
        self.store.purge_expired(plugin).await?;

        let (ticket, token_hash) = self.tickets.generate();
        self.store
            .insert(NewViewTicket {
                token_hash,
                plugin_id: plugin,
                user_id: user.id,
                user_display_name: display_name(user),
                expires_at: Utc::now() + Duration::seconds(TTL_SECONDS),
            })
            .await?;
        Ok(ticket)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn redeem(
        &self,
        plugin: &Plugin,
        ticket: &str,
    ) -> Result<RedeemedTicket, ServiceError> {
        let Some(token_hash) = self.tickets.hash(ticket) else {
            return Err(AuthError::PluginViewTicketInvalid.into());
        };
        self.store
            .redeem(plugin.id, &token_hash)
            .await?
            .ok_or_else(|| AuthError::PluginViewTicketInvalid.into())
    }
}

/// Deliberately no email fallback: the plugin learns who opened the view,
/// not how to contact them.
fn display_name(user: &AuthUser) -> String {
    user.raw_claims
        .get("name")
        .and_then(|n| n.as_str())
        .filter(|n| !n.trim().is_empty())
        .map(str::to_string)
        .or_else(|| user.username.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    };

    use domain::plugin::{Plugin, PluginSnapshot};

    use super::*;

    #[derive(Default)]
    struct MemoryStore {
        rows: Mutex<Vec<(NewViewTicket, bool)>>,
    }

    #[async_trait]
    impl ViewTicketStore for MemoryStore {
        async fn insert(&self, ticket: NewViewTicket) -> Result<(), RepositoryError> {
            self.rows.lock().unwrap().push((ticket, false));
            Ok(())
        }

        async fn redeem(
            &self,
            plugin: Id<Plugin>,
            token_hash: &str,
        ) -> Result<Option<RedeemedTicket>, RepositoryError> {
            let mut rows = self.rows.lock().unwrap();
            let found = rows.iter_mut().find(|(t, redeemed)| {
                !*redeemed
                    && t.token_hash == token_hash
                    && t.plugin_id == plugin
                    && t.expires_at > Utc::now()
            });
            Ok(found.map(|(t, redeemed)| {
                *redeemed = true;
                RedeemedTicket {
                    user_id: t.user_id,
                    user_display_name: t.user_display_name.clone(),
                }
            }))
        }

        async fn purge_expired(&self, _plugin: Id<Plugin>) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct CountingTickets(AtomicU64);

    impl ViewTicketFactory for CountingTickets {
        fn generate(&self) -> (String, String) {
            let ticket = format!("ticket-{}", self.0.fetch_add(1, Ordering::Relaxed));
            let hash = format!("hash:{ticket}");
            (ticket, hash)
        }

        fn hash(&self, ticket: &str) -> Option<String> {
            ticket
                .starts_with("ticket-")
                .then(|| format!("hash:{ticket}"))
        }
    }

    fn service() -> PluginViewTicketService {
        PluginViewTicketService::new(
            Arc::new(MemoryStore::default()),
            Arc::new(CountingTickets::default()),
        )
    }

    fn plugin() -> Plugin {
        Plugin::reconstitute(PluginSnapshot {
            id: Uuid::now_v7(),
            slug: "sensor-setup".into(),
            name: "Sensor Setup".into(),
            description: None,
            frontend_mode: "none".into(),
            frontend_target: None,
            organization_id: Uuid::now_v7(),
            permissions: Vec::new(),
            required_permissions: Vec::new(),
            device_capabilities: Vec::new(),
            enabled: true,
            key_hash: None,
            last_seen_at: None,
        })
    }

    fn user(name: Option<&str>) -> AuthUser {
        AuthUser {
            id: Uuid::now_v7(),
            username: Some("jdoe".into()),
            email: None,
            raw_claims: match name {
                Some(n) => serde_json::json!({ "name": n }),
                None => serde_json::json!({}),
            },
        }
    }

    #[tokio::test]
    async fn issued_ticket_redeems_once_for_its_plugin() {
        let service = service();
        let plugin = plugin();
        let operator = user(Some("Jane Doe"));

        let ticket = service.issue(plugin.id, &operator).await.unwrap();
        let redeemed = service.redeem(&plugin, &ticket).await.unwrap();
        assert_eq!(redeemed.user_id, operator.id);
        assert_eq!(redeemed.user_display_name, "Jane Doe");

        let second = service.redeem(&plugin, &ticket).await;
        assert!(matches!(
            second,
            Err(ServiceError::Auth(AuthError::PluginViewTicketInvalid))
        ));
    }

    #[tokio::test]
    async fn ticket_of_another_plugin_is_rejected() {
        let service = service();
        let ticket = service.issue(plugin().id, &user(None)).await.unwrap();

        let result = service.redeem(&plugin(), &ticket).await;
        assert!(matches!(
            result,
            Err(ServiceError::Auth(AuthError::PluginViewTicketInvalid))
        ));
    }

    #[tokio::test]
    async fn display_name_falls_back_to_username() {
        let service = service();
        let plugin = plugin();
        let ticket = service.issue(plugin.id, &user(None)).await.unwrap();

        let redeemed = service.redeem(&plugin, &ticket).await.unwrap();
        assert_eq!(redeemed.user_display_name, "jdoe");
    }

    #[tokio::test]
    async fn display_name_never_falls_back_to_the_email_address() {
        let service = service();
        let plugin = plugin();
        let operator = AuthUser {
            username: None,
            email: Some("jane.doe@example.org".into()),
            ..user(None)
        };
        let ticket = service.issue(plugin.id, &operator).await.unwrap();

        let redeemed = service.redeem(&plugin, &ticket).await.unwrap();
        assert_eq!(redeemed.user_display_name, "");
    }

    #[tokio::test]
    async fn malformed_ticket_is_rejected_without_store_lookup() {
        let service = service();
        let result = service.redeem(&plugin(), "gep_not-a-ticket").await;
        assert!(matches!(
            result,
            Err(ServiceError::Auth(AuthError::PluginViewTicketInvalid))
        ));
    }
}
