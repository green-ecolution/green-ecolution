use std::sync::{Arc, Weak};

use domain::{Id, RepositoryError, events::DomainEvent};
use irrigation_domain::{
    events::IrrigationEvent,
    tree::{TreeReader, TreeWriter},
};

use crate::irrigation::targets::{is_tree, tree_target};
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
            _ => match IrrigationEvent::from_domain(event) {
                Some(IrrigationEvent::TreeDeleted { tree_id, .. }) => {
                    // Nothing else frees the sensor of a deleted tree, so a lost
                    // service must surface instead of passing silently.
                    let Some(sensors) = self.sensors.upgrade() else {
                        return Err(EventHandlerError::Failed(
                            "sensor service is gone; sensor of the deleted tree stays attached"
                                .into(),
                        ));
                    };
                    sensors
                        .release_target(&tree_target(*tree_id))
                        .await
                        .map_err(|e| EventHandlerError::Failed(e.to_string()))?;
                    Ok(vec![])
                }
                _ => Ok(vec![]),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sqlx::postgres::PgPoolOptions;

    use super::*;
    use crate::{
        infra::{
            pg_module_activation::PgModuleActivationRepository,
            pg_organization::PgOrganizationRepository, pg_sensor::PgSensorRepository,
            pg_sensor_model::PgSensorModelRepository,
        },
        irrigation::infra::pg_tree::PgTreeRepository,
        service::{event_bus::DeferredEventBus, targets::TargetRegistry},
    };

    #[tokio::test]
    async fn a_tree_deleted_after_the_sensor_service_is_gone_is_an_error() {
        // Lazy: nothing here reaches the database.
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://unused@127.0.0.1:1/unused")
            .unwrap();
        let trees = Arc::new(PgTreeRepository::new(pool.clone()));
        let sensors = Arc::new(PgSensorRepository::new(
            pool.clone(),
            chrono::Duration::hours(24),
            3,
        ));
        let service = Arc::new(SensorService::new(
            sensors.clone(),
            sensors.clone(),
            sensors.clone(),
            sensors,
            Arc::new(PgSensorModelRepository::new(pool.clone())),
            Arc::new(TargetRegistry::new()),
            Arc::new(PgModuleActivationRepository::new(
                pool.clone(),
                Arc::new(PgOrganizationRepository::new(pool)),
                BTreeMap::new(),
            )),
            Arc::new(DeferredEventBus::new()),
        ));
        let handler = SensorLifecycle::new(trees.clone(), trees, &service);
        drop(service);

        let result = handler
            .handle(
                &IrrigationEvent::TreeDeleted {
                    tree_id: Id::new_v7(),
                    cluster_id: None,
                }
                .into_domain(),
            )
            .await;

        assert!(matches!(result, Err(EventHandlerError::Failed(_))));
    }
}
