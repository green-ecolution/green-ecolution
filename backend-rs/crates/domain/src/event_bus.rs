use crate::RepositoryError;
use crate::events::DomainEvent;

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
