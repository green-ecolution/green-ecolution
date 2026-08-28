use crate::helpers::{TestApp, spawn_app};
use uuid::Uuid;

async fn insert_sensor(app: &TestApp, id: &str, activated: bool) {
    let model_id = app.ecodrizzler_model_id().await;
    let activated_at = activated.then(|| chrono::Utc::now().naive_utc());
    sqlx::query!(
        r#"INSERT INTO sensors (id, activated_at, type, model_id, organization_id)
        VALUES ($1, $2, 'lorawan', $3, '01980000-0000-7000-8000-000000000001')"#,
        id,
        activated_at,
        model_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
        VALUES ($1, '', '', '', '')"#,
        id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_reading(app: &TestApp, sensor_id: &str, reading_id: Uuid) {
    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, $3)"#,
        reading_id,
        sensor_id,
        serde_json::json!({"temperature": 20.0}),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

/// UUIDv7 whose embedded timestamp (2024-01-01T00:00:00Z) is far past the
/// 24 h default threshold.
fn stale_reading_id() -> Uuid {
    Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        1_704_067_200,
        0,
    ))
}

async fn status_of(app: &TestApp, id: &str) -> serde_json::Value {
    let body: serde_json::Value = app
        .get(&format!("/api/v1/sensors/{id}"))
        .await
        .json()
        .await
        .unwrap();
    body["status"].clone()
}

async fn status_in_list(app: &TestApp, id: &str) -> serde_json::Value {
    let body: serde_json::Value = app.get("/api/v1/sensors").await.json().await.unwrap();
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id)
        .expect("sensor present in list")["status"]
        .clone()
}

#[tokio::test]
async fn activated_sensor_with_fresh_reading_is_online() {
    let app = spawn_app().await;
    insert_sensor(&app, "sensor-on", true).await;
    insert_reading(&app, "sensor-on", Uuid::now_v7()).await;

    assert_eq!(status_of(&app, "sensor-on").await, "online");
    assert_eq!(status_in_list(&app, "sensor-on").await, "online");
}

#[tokio::test]
async fn activated_sensor_with_stale_reading_is_offline() {
    let app = spawn_app().await;
    insert_sensor(&app, "sensor-stale", true).await;
    insert_reading(&app, "sensor-stale", stale_reading_id()).await;

    assert_eq!(status_of(&app, "sensor-stale").await, "offline");
    assert_eq!(status_in_list(&app, "sensor-stale").await, "offline");
}

#[tokio::test]
async fn activated_sensor_without_reading_is_offline() {
    let app = spawn_app().await;
    insert_sensor(&app, "sensor-silent", true).await;

    assert_eq!(status_of(&app, "sensor-silent").await, "offline");
    assert_eq!(status_in_list(&app, "sensor-silent").await, "offline");
}

#[tokio::test]
async fn prepared_sensor_with_fresh_reading_stays_prepared() {
    let app = spawn_app().await;
    insert_sensor(&app, "sensor-prep", false).await;
    insert_reading(&app, "sensor-prep", Uuid::now_v7()).await;

    assert_eq!(status_of(&app, "sensor-prep").await, "prepared");
    assert_eq!(status_in_list(&app, "sensor-prep").await, "prepared");
}

async fn soil_tension_ability_id(app: &TestApp, depth_cm: i32) -> Uuid {
    sqlx::query_scalar!(
        r#"SELECT sma.id
           FROM sensor_model_abilities sma
           JOIN sensor_models m ON m.id = sma.sensor_model_id AND m.name = 'EcoDrizzler'
           JOIN sensor_abilities sa ON sa.id = sma.sensor_ability_id
           WHERE sa.ability = 'soil_tension' AND sma.depth_cm = $1"#,
        depth_cm,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

/// One uplink carrying a single soil-tension value with an explicit verdict.
async fn insert_uplink(app: &TestApp, sensor_id: &str, value: f64, plausible: bool) {
    let data_id = Uuid::now_v7();
    let ability_id = soil_tension_ability_id(app, 30).await;
    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, '{}'::jsonb)"#,
        data_id,
        sensor_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO sensor_data_ability_values
             (sensor_data_id, sensor_model_ability_id, value, plausible, quality_reason)
           VALUES ($1, $2, $3::float8::numeric, $4, CASE WHEN $4 THEN NULL ELSE 'out_of_range' END)"#,
        data_id,
        ability_id,
        value,
        plausible,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn three_faulty_uplinks_mark_the_sensor_as_suspect() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-suspect", true).await;

    let body: serde_json::Value = app
        .get("/api/v1/sensors/eui-suspect")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["data_health"], "ok");
    assert_eq!(body["implausible_recent"], 0);

    for _ in 0..3 {
        insert_uplink(&app, "eui-suspect", 6553.5, false).await;
    }

    let body: serde_json::Value = app
        .get("/api/v1/sensors/eui-suspect")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["data_health"], "suspect");
    assert_eq!(body["implausible_recent"], 3);
}

#[tokio::test]
async fn one_plausible_uplink_clears_the_suspicion() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-cleared", true).await;

    for _ in 0..3 {
        insert_uplink(&app, "eui-cleared", 6553.5, false).await;
    }
    insert_uplink(&app, "eui-cleared", 45.0, true).await;

    let body: serde_json::Value = app
        .get("/api/v1/sensors/eui-cleared")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["data_health"], "ok");
    assert_eq!(body["implausible_recent"], 3);
}

#[tokio::test]
async fn data_health_is_reported_in_the_sensor_list() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-list-health", true).await;
    for _ in 0..3 {
        insert_uplink(&app, "eui-list-health", 6553.5, false).await;
    }

    let body: serde_json::Value = app.get("/api/v1/sensors").await.json().await.unwrap();
    let entry = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "eui-list-health")
        .expect("sensor present in list");
    assert_eq!(entry["data_health"], "suspect");
}
