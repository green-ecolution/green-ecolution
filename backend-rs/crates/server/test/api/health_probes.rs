use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use domain::info::{ServiceMessage, ServiceName};
use server::infra::health::keycloak_probe::KeycloakProbe;
use server::infra::health::mqtt_probe::MqttProbe;
use server::infra::health::pg_probe::PgProbe;
use server::infra::health::probe::HealthProbe;
use server::infra::mqtt::MqttHealthState;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::helpers::spawn_app;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

#[tokio::test]
async fn pg_probe_healthy_on_live_pool() {
    let app = spawn_app().await;
    let probe = PgProbe::new(app.db_pool.clone());
    let status = probe.check().await;

    assert_eq!(probe.name(), ServiceName::Postgres);
    assert!(status.enabled);
    assert!(status.healthy);
    assert_eq!(status.message, ServiceMessage::Connected);
}

#[tokio::test]
async fn pg_probe_unhealthy_on_closed_pool() {
    let app = spawn_app().await;
    let probe = PgProbe::new(app.db_pool.clone());
    app.db_pool.close().await;
    let status = probe.check().await;

    assert!(!status.healthy);
    assert_eq!(status.message, ServiceMessage::NoConnection);
}

#[tokio::test]
async fn keycloak_probe_disabled_reports_disabled_message() {
    let probe = KeycloakProbe::new(false, None, http_client(), Duration::from_secs(1));
    let status = probe.check().await;
    assert!(!status.enabled);
    assert_eq!(status.message, ServiceMessage::Disabled);
}

#[tokio::test]
async fn keycloak_probe_no_issuer_reports_not_configured() {
    let probe = KeycloakProbe::new(true, None, http_client(), Duration::from_secs(1));
    let status = probe.check().await;
    assert!(status.enabled);
    assert!(!status.healthy);
    assert_eq!(status.message, ServiceMessage::NotConfigured);
}

#[tokio::test]
async fn keycloak_probe_healthy_when_well_known_returns_200() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/realms/example/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"issuer": "x"})))
        .mount(&mock)
        .await;

    let probe = KeycloakProbe::new(
        true,
        Some(&format!("{}/realms/example", mock.uri())),
        http_client(),
        Duration::from_secs(2),
    );
    let status = probe.check().await;
    assert!(status.healthy);
    assert_eq!(status.message, ServiceMessage::Connected);
}

#[tokio::test]
async fn keycloak_probe_unhealthy_on_500() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/realms/example/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;

    let probe = KeycloakProbe::new(
        true,
        Some(&format!("{}/realms/example", mock.uri())),
        http_client(),
        Duration::from_secs(2),
    );
    let status = probe.check().await;
    assert!(!status.healthy);
    assert_eq!(status.message, ServiceMessage::ConnectionError);
}

#[tokio::test]
async fn keycloak_probe_handles_trailing_slash_in_issuer() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/realms/example/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"issuer": "x"})))
        .mount(&mock)
        .await;

    let probe = KeycloakProbe::new(
        true,
        Some(&format!("{}/realms/example/", mock.uri())),
        http_client(),
        Duration::from_secs(2),
    );
    let status = probe.check().await;
    assert!(status.healthy);
}

#[tokio::test]
async fn mqtt_probe_disabled() {
    let state = Arc::new(MqttHealthState::default());
    let probe = MqttProbe::new(false, state);
    let status = probe.check().await;
    assert!(!status.enabled);
    assert_eq!(status.message, ServiceMessage::Disabled);
}

#[tokio::test]
async fn mqtt_probe_reports_state_flag() {
    let state = Arc::new(MqttHealthState::default());
    state.connected.store(true, Ordering::Relaxed);
    let probe = MqttProbe::new(true, state.clone());
    let status = probe.check().await;
    assert!(status.healthy);
    assert_eq!(status.message, ServiceMessage::Connected);

    state.connected.store(false, Ordering::Relaxed);
    let status = probe.check().await;
    assert!(!status.healthy);
    assert_eq!(status.message, ServiceMessage::NoConnection);
}

use domain::info::HealthSnapshotReader;
use server::infra::health::spawn as spawn_health;

struct FakeProbe {
    name: ServiceName,
    healthy: bool,
}

#[async_trait::async_trait]
impl HealthProbe for FakeProbe {
    fn name(&self) -> ServiceName {
        self.name
    }
    async fn check(&self) -> domain::info::ServiceStatus {
        domain::info::ServiceStatus {
            name: self.name,
            enabled: true,
            healthy: self.healthy,
            response_time: Duration::ZERO,
            last_checked: chrono::Utc::now(),
            message: if self.healthy {
                ServiceMessage::Connected
            } else {
                ServiceMessage::NoConnection
            },
        }
    }
}

#[tokio::test]
async fn health_coordinator_aggregates_probes_after_first_tick() {
    let probes: Vec<Arc<dyn HealthProbe>> = vec![
        Arc::new(FakeProbe { name: ServiceName::Postgres, healthy: true }),
        Arc::new(FakeProbe { name: ServiceName::Keycloak, healthy: false }),
    ];
    let (coord, handle) = spawn_health(probes, Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(150)).await;

    let snapshot = coord.snapshot().await;
    assert_eq!(snapshot.len(), 2);
    assert!(snapshot.iter().any(|s| s.name == ServiceName::Postgres && s.healthy));
    assert!(snapshot.iter().any(|s| s.name == ServiceName::Keycloak && !s.healthy));

    handle.abort();
}
