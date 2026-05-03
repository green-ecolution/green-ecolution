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

    /// Publishes a batch of events sequentially. Default implementation
    /// iterates over `publish`; concrete buses may override for batching.
    async fn publish_all(&self, events: Vec<DomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
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
        for handler in self.handlers.iter().filter(|h| h.handles(&event)) {
            if let Err(e) = handler.handle(&event).await {
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
