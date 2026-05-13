use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use url::Url;

use domain::info::{ServiceMessage, ServiceName, ServiceStatus};

use super::probe::HealthProbe;

pub struct KeycloakProbe {
    enabled: bool,
    well_known_url: Option<Url>,
    client: Client,
    timeout: Duration,
}

impl KeycloakProbe {
    pub fn new(enabled: bool, issuer_url: Option<&str>, client: Client, timeout: Duration) -> Self {
        let well_known_url = issuer_url.and_then(|s| {
            let trimmed = s.trim_end_matches('/');
            Url::parse(&format!("{trimmed}/.well-known/openid-configuration")).ok()
        });
        Self { enabled, well_known_url, client, timeout }
    }
}

#[async_trait]
impl HealthProbe for KeycloakProbe {
    fn name(&self) -> ServiceName {
        ServiceName::Keycloak
    }

    async fn check(&self) -> ServiceStatus {
        let now = Utc::now();
        if !self.enabled {
            return ServiceStatus {
                name: ServiceName::Keycloak,
                enabled: false,
                healthy: false,
                response_time: Duration::ZERO,
                last_checked: now,
                message: ServiceMessage::Disabled,
            };
        }
        let Some(url) = &self.well_known_url else {
            return ServiceStatus {
                name: ServiceName::Keycloak,
                enabled: true,
                healthy: false,
                response_time: Duration::ZERO,
                last_checked: now,
                message: ServiceMessage::NotConfigured,
            };
        };

        let start = Instant::now();
        let result = self
            .client
            .get(url.clone())
            .timeout(self.timeout)
            .send()
            .await
            .and_then(|r| r.error_for_status());

        let elapsed = start.elapsed();
        let healthy = result.is_ok();
        ServiceStatus {
            name: ServiceName::Keycloak,
            enabled: true,
            healthy,
            response_time: elapsed,
            last_checked: now,
            message: if healthy {
                ServiceMessage::Connected
            } else {
                ServiceMessage::ConnectionError
            },
        }
    }
}
