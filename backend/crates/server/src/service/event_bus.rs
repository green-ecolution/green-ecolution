use std::sync::{Arc, OnceLock};

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

#[derive(Debug, thiserror::Error)]
#[error("the event bus is already installed")]
pub struct BusAlreadyInstalled;

/// Services need a bus when they are built, modules contribute handlers only
/// afterwards. This handle is passed around first and receives the real bus
/// once every handler is known.
#[derive(Default)]
pub struct DeferredEventBus {
    inner: OnceLock<Arc<dyn EventBus>>,
}

impl DeferredEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(&self, bus: Arc<dyn EventBus>) -> Result<(), BusAlreadyInstalled> {
        self.inner.set(bus).map_err(|_| BusAlreadyInstalled)
    }
}

#[async_trait::async_trait]
impl EventBus for DeferredEventBus {
    async fn publish(&self, event: DomainEvent) {
        match self.inner.get() {
            Some(bus) => bus.publish(event).await,
            None => {
                tracing::error!(event = ?event, "event published before the bus was installed; dropped")
            }
        }
    }
}

pub struct NoopEventBus;

#[async_trait::async_trait]
impl EventBus for NoopEventBus {
    async fn publish(&self, _event: DomainEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Recording(Mutex<Vec<String>>);

    #[async_trait::async_trait]
    impl EventBus for Recording {
        async fn publish(&self, event: DomainEvent) {
            self.0.lock().unwrap().push(format!("{event:?}"));
        }
    }

    fn event() -> DomainEvent {
        DomainEvent::OrganizationDeleted {
            organization_id: domain::Id::new_v7(),
        }
    }

    #[tokio::test]
    async fn forwards_once_installed() {
        let deferred = DeferredEventBus::new();
        let inner = Arc::new(Recording(Mutex::new(Vec::new())));
        deferred.install(inner.clone()).unwrap();
        deferred.publish(event()).await;
        assert_eq!(inner.0.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn drops_events_before_installation() {
        let deferred = DeferredEventBus::new();
        deferred.publish(event()).await;
        let inner = Arc::new(Recording(Mutex::new(Vec::new())));
        deferred.install(inner.clone()).unwrap();
        assert!(inner.0.lock().unwrap().is_empty());
    }

    #[test]
    fn installs_only_once() {
        let deferred = DeferredEventBus::new();
        deferred.install(Arc::new(NoopEventBus)).unwrap();
        assert!(deferred.install(Arc::new(NoopEventBus)).is_err());
    }
}
