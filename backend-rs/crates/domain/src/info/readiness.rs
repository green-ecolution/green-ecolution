use super::ServiceStatus;

#[derive(Debug, Clone)]
pub struct Readiness {
    pub services: Vec<ServiceStatus>,
}

impl Readiness {
    pub fn new(services: Vec<ServiceStatus>) -> Self {
        Self { services }
    }

    pub fn is_ready(&self) -> bool {
        self.services.iter().all(|s| s.healthy)
    }
}

#[async_trait::async_trait]
pub trait ReadinessReader: Send + Sync {
    async fn readiness(&self) -> Readiness;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;

    use crate::info::readiness::Readiness;
    use crate::info::{ServiceMessage, ServiceName, ServiceStatus};

    fn status(name: ServiceName, healthy: bool) -> ServiceStatus {
        ServiceStatus {
            name,
            enabled: true,
            healthy,
            response_time: Duration::ZERO,
            last_checked: Utc::now(),
            message: if healthy {
                ServiceMessage::Connected
            } else {
                ServiceMessage::NoConnection
            },
        }
    }

    #[test]
    fn ready_when_all_critical_services_healthy() {
        let readiness = Readiness::new(vec![status(ServiceName::Postgres, true)]);
        assert!(readiness.is_ready());
    }

    #[test]
    fn not_ready_when_any_critical_service_unhealthy() {
        let readiness = Readiness::new(vec![
            status(ServiceName::Postgres, false),
            status(ServiceName::Keycloak, true),
        ]);
        assert!(!readiness.is_ready());
    }
}
