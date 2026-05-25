//! Static probe that reports a feature flag's enabled state.
//!
//! Unlike `PgProbe` / `KeycloakProbe`, no network roundtrip happens —
//! the probe simply mirrors the configured flag so `/info/services`
//! lists feature-gated subsystems alongside real services.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use domain::info::{ServiceMessage, ServiceName, ServiceStatus};

use super::probe::HealthProbe;

pub struct FeatureProbe {
    name: ServiceName,
    enabled: bool,
}

impl FeatureProbe {
    pub fn new(name: ServiceName, enabled: bool) -> Self {
        Self { name, enabled }
    }
}

#[async_trait]
impl HealthProbe for FeatureProbe {
    fn name(&self) -> ServiceName {
        self.name
    }

    async fn check(&self) -> ServiceStatus {
        ServiceStatus {
            name: self.name,
            enabled: self.enabled,
            healthy: self.enabled,
            response_time: Duration::ZERO,
            last_checked: Utc::now(),
            message: if self.enabled {
                ServiceMessage::Connected
            } else {
                ServiceMessage::Disabled
            },
        }
    }
}
