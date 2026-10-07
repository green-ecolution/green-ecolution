use serde_json::json;
use uuid::Uuid;

use crate::helpers::{install_plugin, spawn_app_with_plugins};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

fn sensor_body(dev_eui: &str, model_id: Uuid) -> serde_json::Value {
    json!({
        "id": format!("eui-{dev_eui}"),
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "LA665932558",
            "dev_eui": dev_eui,
            "app_eui": "a840410000000101",
            "app_key": "00112233445566778899aabbccddeeff",
            "at_pin": "1a2b3c4d",
            "ota_pin": "5e6f7a8b"
        }
    })
}

#[tokio::test]
async fn plugin_creates_a_prepared_sensor_in_its_organization() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "sensor-setup", &["sensor:read", "sensor:create"]).await;

    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/sensors",
            &sensor_body("a84041000181c001", app.ges_1000_model_id().await),
            &key,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "prepared");

    let row = sqlx::query!(
        "SELECT organization_id, provider FROM sensors WHERE id = $1",
        "eui-a84041000181c001"
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(row.organization_id.to_string(), ROOT_ORG);
    assert_eq!(row.provider.as_deref(), Some("sensor-setup"));
}

#[tokio::test]
async fn sensor_ingest_without_sensor_create_is_forbidden() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "reader", &["sensor:read"]).await;

    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/sensors",
            &sensor_body("a84041000181c002", app.ges_1000_model_id().await),
            &key,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[tokio::test]
async fn duplicate_sensor_is_a_conflict() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "sensor-setup", &["sensor:create"]).await;
    let body = sensor_body("a84041000181c003", app.ges_1000_model_id().await);

    let first = app
        .post_json_with_bearer("/api/v1/plugins/ingest/sensors", &body, &key)
        .await;
    assert_eq!(first.status().as_u16(), 201);
    let second = app
        .post_json_with_bearer("/api/v1/plugins/ingest/sensors", &body, &key)
        .await;
    assert_eq!(second.status().as_u16(), 409);
}

#[tokio::test]
async fn body_cannot_choose_organization_or_provider() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "sensor-setup", &["sensor:create"]).await;
    let mut body = sensor_body("a84041000181c004", app.ges_1000_model_id().await);
    body["organization_id"] = json!("01980000-0000-7000-8000-000000000003");
    body["provider"] = json!("someone-else");

    let resp = app
        .post_json_with_bearer("/api/v1/plugins/ingest/sensors", &body, &key)
        .await;
    assert_eq!(resp.status().as_u16(), 201);

    let row = sqlx::query!(
        "SELECT organization_id, provider FROM sensors WHERE id = $1",
        "eui-a84041000181c004"
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(row.organization_id.to_string(), ROOT_ORG);
    assert_eq!(row.provider.as_deref(), Some("sensor-setup"));
}

#[tokio::test]
async fn sensor_models_need_sensor_read() {
    let app = spawn_app_with_plugins().await;
    let reader = install_plugin(&app, "reader", &["sensor:read"]).await;
    let blind = install_plugin(&app, "blind", &["tree:read"]).await;

    let ok = app
        .get_with_bearer("/api/v1/plugins/ingest/sensor-models", &reader)
        .await;
    assert_eq!(ok.status().as_u16(), 200);
    let models: serde_json::Value = ok.json().await.unwrap();
    assert!(
        models
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["name"] == "GES-1000")
    );

    let denied = app
        .get_with_bearer("/api/v1/plugins/ingest/sensor-models", &blind)
        .await;
    assert_eq!(denied.status().as_u16(), 403);
}
