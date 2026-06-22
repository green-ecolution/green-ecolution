use std::sync::Arc;

pub use domain::event_bus::{EventBus, EventHandler, EventHandlerError};
use domain::events::DomainEvent;

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
            for handler in &self.handlers {
                match handler.handle(&event).await {
                    Ok(follow_ups) => queue.extend(follow_ups),
                    Err(e) => tracing::error!(
                        event.handler = handler.name(),
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
