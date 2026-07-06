use std::sync::Arc;

use async_trait::async_trait;
use domain::info::{Readiness, ReadinessReader};

use super::probe::HealthProbe;

/// Keep the probe list to dependencies whose loss must remove the pod from the
/// load balancer (the DB). Gating readiness on optional services like Keycloak
/// or MQTT would cascade their outages across every pod.
pub struct DefaultReadiness {
    critical: Vec<Arc<dyn HealthProbe>>,
}

impl DefaultReadiness {
    pub fn new(critical: Vec<Arc<dyn HealthProbe>>) -> Self {
        Self { critical }
    }
}

#[async_trait]
impl ReadinessReader for DefaultReadiness {
    async fn readiness(&self) -> Readiness {
        let services = futures::future::join_all(self.critical.iter().map(|p| p.check())).await;
        Readiness::new(services)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::Utc;
    use domain::info::{ReadinessReader, ServiceMessage, ServiceName, ServiceStatus};

    use super::DefaultReadiness;
    use crate::infra::health::probe::HealthProbe;

    struct FakeProbe {
        name: ServiceName,
        healthy: bool,
    }

    #[async_trait]
    impl HealthProbe for FakeProbe {
        fn name(&self) -> ServiceName {
            self.name
        }

        async fn check(&self) -> ServiceStatus {
            ServiceStatus {
                name: self.name,
                enabled: true,
                healthy: self.healthy,
                response_time: Duration::ZERO,
                last_checked: Utc::now(),
                message: if self.healthy {
                    ServiceMessage::Connected
                } else {
                    ServiceMessage::NoConnection
                },
            }
        }
    }

    fn probe(name: ServiceName, healthy: bool) -> Arc<dyn HealthProbe> {
        Arc::new(FakeProbe { name, healthy })
    }

    #[tokio::test]
    async fn ready_when_all_critical_probes_healthy() {
        let reader = DefaultReadiness::new(vec![probe(ServiceName::Postgres, true)]);
        assert!(reader.readiness().await.is_ready());
    }

    #[tokio::test]
    async fn not_ready_when_a_critical_probe_unhealthy() {
        let reader = DefaultReadiness::new(vec![probe(ServiceName::Postgres, false)]);
        assert!(!reader.readiness().await.is_ready());
    }
}
