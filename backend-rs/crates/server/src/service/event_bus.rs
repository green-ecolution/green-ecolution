use std::sync::Arc;

use domain::{RepositoryError, events::DomainEvent};

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
    /// Reacts to `event` and returns any follow-up events the handler wants
    /// the bus to publish next. Returning `Ok(vec![])` is the common case
    /// (handler had a side effect but did not produce new domain events).
    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError>;
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
        let mut queue: Vec<DomainEvent> = vec![event];
        while let Some(event) = queue.pop() {
            for handler in self.handlers.iter().filter(|h| h.handles(&event)) {
                match handler.handle(&event).await {
                    Ok(follow_ups) => queue.extend(follow_ups),
                    Err(e) => tracing::error!(
                        handler = handler.name(),
                        error = %e,
                        event = ?event,
                        "event handler failed"
                    ),
                }
            }
        }
    }
}

pub struct NoopEventBus;

#[async_trait::async_trait]
impl EventBus for NoopEventBus {
    async fn publish(&self, _event: DomainEvent) {}
}
