use std::time::Duration;

use domain::info::{ServiceMessage, ServiceName};
use server::infra::health::keycloak_probe::KeycloakProbe;
use server::infra::health::pg_probe::PgProbe;
use server::infra::health::probe::HealthProbe;
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
