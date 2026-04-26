use std::sync::Arc;

use crate::domain::{RepositoryError, events::DomainEvent};

#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("handler failed: {0}")]
    Failed(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    fn name(&self) -> &str;
    fn handles(&self, event: &DomainEvent) -> bool;
    async fn handle(&self, event: &DomainEvent) -> Result<(), EventHandlerError>;
}

#[async_trait::async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: DomainEvent);
}

pub struct InMemoryEventBus {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl InMemoryEventBus {
    pub fn new(handlers: Vec<Arc<dyn EventHandler>>) -> Self {
        Self { handlers }
    }
}

#[async_trait::async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: DomainEvent) {
        let applicable: Vec<_> = self
            .handlers
            .iter()
            .filter(|h| h.handles(&event))
            .collect();

        let results = futures::future::join_all(
            applicable.iter().map(|h| h.handle(&event)),
        )
        .await;

        for (handler, result) in applicable.iter().zip(results) {
            if let Err(e) = result {
                tracing::error!(
                    handler = handler.name(),
                    error = %e,
                    event = ?event,
                    "event handler failed"
                );
            }
        }
    }
}

pub struct NoopEventBus;

#[async_trait::async_trait]
impl EventBus for NoopEventBus {
    async fn publish(&self, _event: DomainEvent) {}
}
