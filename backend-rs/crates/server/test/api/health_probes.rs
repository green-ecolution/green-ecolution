use domain::info::{ServiceMessage, ServiceName};
use server::infra::health::pg_probe::PgProbe;
use server::infra::health::probe::HealthProbe;

use crate::helpers::spawn_app;

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
