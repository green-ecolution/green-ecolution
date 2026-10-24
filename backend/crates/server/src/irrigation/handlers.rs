use std::sync::{Arc, Weak};

use domain::{
    Id, RepositoryError,
    events::DomainEvent,
    tree::{TreeReader, TreeWriter},
};

use super::targets::{is_tree, tree_target};
use crate::service::{
    event_bus::{EventHandler, EventHandlerError},
    sensor_service::SensorService,
};

/// A tree without a sensor shows no measured status; a deleted tree frees its sensor.
pub struct SensorLifecycle {
    trees: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    // The service publishes on the bus that owns this handler; a strong
    // reference would close that cycle and keep the bus and its pool alive.
    sensors: Weak<SensorService>,
}

impl SensorLifecycle {
    pub fn new(
        trees: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        sensors: &Arc<SensorService>,
    ) -> Self {
        Self {
            trees,
            tree_writer,
            sensors: Arc::downgrade(sensors),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for SensorLifecycle {
    fn name(&self) -> &str {
        "irrigation_sensor_lifecycle"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        match event {
            DomainEvent::SensorDetached { target, .. } if is_tree(target) => {
                let mut tree = match self.trees.by_id(Id::new(target.id)).await {
                    Ok(tree) => tree,
                    Err(RepositoryError::NotFound) => return Ok(vec![]),
                    Err(e) => return Err(e.into()),
                };
                let events = tree.forget_watering_status();
                if events.is_empty() {
                    return Ok(vec![]);
                }
                self.tree_writer.save(&tree).await?;
                Ok(events)
            }
            DomainEvent::TreeDeleted { tree_id, .. } => {
                let Some(sensors) = self.sensors.upgrade() else {
                    return Ok(vec![]);
                };
                sensors
                    .release_target(&tree_target(*tree_id))
                    .await
                    .map_err(|e| EventHandlerError::Failed(e.to_string()))?;
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}
