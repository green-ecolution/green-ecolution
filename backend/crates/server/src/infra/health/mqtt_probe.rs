use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use domain::info::{ServiceMessage, ServiceName, ServiceStatus};

use crate::infra::mqtt::MqttHealthState;

use super::probe::HealthProbe;

pub struct MqttProbe {
    enabled: bool,
    state: Arc<MqttHealthState>,
}

impl MqttProbe {
    pub fn new(enabled: bool, state: Arc<MqttHealthState>) -> Self {
        Self { enabled, state }
    }
}

#[async_trait]
impl HealthProbe for MqttProbe {
    fn name(&self) -> ServiceName {
        ServiceName::Mqtt
    }

    async fn check(&self) -> ServiceStatus {
        let now = Utc::now();
        if !self.enabled {
            return ServiceStatus {
                name: ServiceName::Mqtt,
                enabled: false,
                healthy: false,
                response_time: Duration::ZERO,
                last_checked: now,
                message: ServiceMessage::Disabled,
            };
        }
        let healthy = self.state.connected.load(Ordering::Relaxed);
        ServiceStatus {
            name: ServiceName::Mqtt,
            enabled: true,
            healthy,
            response_time: Duration::ZERO,
            last_checked: now,
            message: if healthy {
                ServiceMessage::Connected
            } else {
                ServiceMessage::NoConnection
            },
        }
    }
}
