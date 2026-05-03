use crate::helpers::spawn_app;

async fn insert_sensor(app: &crate::helpers::TestApp, id: &str) {
    sqlx::query!(
        r#"INSERT INTO sensors (id, status, latitude, longitude, geometry)
        VALUES ($1, 'online', 53.55, 9.99, ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326))"#,
        id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn list_sensors_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/sensors").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_sensors_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/sensors").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_sensors_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/sensors/nonexistent").await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn list_sensors_returns_inserted_sensors() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-001").await;
    insert_sensor(&app, "sensor-002").await;

    let response = app.get("/api/v1/sensors").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 2);
}

#[tokio::test]
async fn get_sensor_returns_full_response() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-100").await;

    let response = app.get("/api/v1/sensors/sensor-100").await;

    assert_eq!(response.status().as_u16(), 200);

    let sensor: serde_json::Value = response.json().await.unwrap();
    assert_eq!(sensor["id"], "sensor-100");
    assert_eq!(sensor["status"], "online");
    assert_eq!(sensor["latitude"], 53.55);
    assert_eq!(sensor["longitude"], 9.99);
}

#[tokio::test]
async fn delete_sensor_returns_204() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-del").await;

    let response = app.delete("/api/v1/sensors/sensor-del").await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get("/api/v1/sensors/sensor-del").await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn list_sensor_data_returns_empty_for_sensor_without_data() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-nodata").await;

    let response = app.get("/api/v1/sensors/sensor-nodata/data").await;

    assert_eq!(response.status().as_u16(), 200);

    let data: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(data.is_empty());
}

#[tokio::test]
async fn list_sensor_data_returns_inserted_data() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-data").await;

    sqlx::query!(
        r#"INSERT INTO sensor_data (sensor_id, data) VALUES ($1, $2)"#,
        "sensor-data",
        serde_json::json!({"temperature": 22.5})
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.get("/api/v1/sensors/sensor-data/data").await;

    assert_eq!(response.status().as_u16(), 200);

    let data: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["data"]["temperature"], 22.5);
}

#[tokio::test]
async fn list_sensors_respects_pagination() {
    let app = spawn_app().await;

    for i in 1..=5 {
        insert_sensor(&app, &format!("sensor-pg-{:03}", i)).await;
    }

    let response = app.get("/api/v1/sensors?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
}

#[tokio::test]
async fn delete_sensor_unlinks_from_tree() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-unlink").await;

    sqlx::query!(
        r#"INSERT INTO trees (sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description)
        VALUES ('sensor-unlink', 2020, 'Eiche', 'T-UNL', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test')"#,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.delete("/api/v1/sensors/sensor-unlink").await;
    assert_eq!(response.status().as_u16(), 204);

    let tree_sensor: Option<String> =
        sqlx::query_scalar!("SELECT sensor_id FROM trees WHERE number = 'T-UNL'")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert!(tree_sensor.is_none());
}

#[tokio::test]
async fn get_tree_by_sensor_returns_linked_tree() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-tree").await;
    sqlx::query!(
        r#"INSERT INTO trees (sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description)
        VALUES ('sensor-tree', 2020, 'Eiche', 'T-LINK', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test')"#,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.get("/api/v1/sensors/sensor-tree/tree").await;
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["number"], "T-LINK");
    assert_eq!(body["sensor"]["id"], "sensor-tree");
}

#[tokio::test]
async fn get_tree_by_sensor_returns_404_for_unknown_sensor() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/sensors/does-not-exist/tree").await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_tree_by_sensor_returns_404_when_no_tree_linked() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-orphan").await;

    let response = app.get("/api/v1/sensors/sensor-orphan/tree").await;

    assert_eq!(response.status().as_u16(), 404);
}

// -- MQTT ingest path: handle_message → auto-create + map-to-tree + watering --

fn payload(device: &str, lat: f64, lng: f64, centibar: i32) -> serde_json::Value {
    serde_json::json!({
        "device": device,
        "battery": 3.6,
        "humidity": 0.4,
        "temperature": 18.5,
        "latitude": lat,
        "longitude": lng,
        "watermarks": [
            {"depth": 30, "resistance": 0, "centibar": centibar},
            {"depth": 60, "resistance": 0, "centibar": centibar},
            {"depth": 90, "resistance": 0, "centibar": centibar},
        ]
    })
}

#[tokio::test]
async fn handle_message_creates_sensor_links_nearest_tree_and_updates_watering_status() {
    let app = spawn_app().await;

    sqlx::query!(
        r#"INSERT INTO trees (planting_year, species, number, latitude, longitude,
                              geometry, description)
        VALUES ($1, 'Eiche', 'T-MQ-1', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test')"#,
        chrono::Utc::now().date_naive().format("%Y").to_string().parse::<i32>().unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    // 50 centibar with 0/1-year defaults (lower=25, higher=33) → score=2 → Bad.
    app.handle_mqtt_message(payload("sensor-mq-1", 53.55, 9.99, 50))
        .await
        .expect("handle_message should succeed");

    let sensor_resp = app.get("/api/v1/sensors/sensor-mq-1").await;
    assert_eq!(sensor_resp.status().as_u16(), 200);

    let tree_status: String = sqlx::query_scalar!(
        r#"SELECT watering_status::text AS "ws!" FROM trees WHERE number = 'T-MQ-1'"#,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(
        tree_status, "bad",
        "tree at sensor's coordinate should be linked and watered status updated to bad"
    );

    let linked_sensor: Option<String> =
        sqlx::query_scalar!("SELECT sensor_id FROM trees WHERE number = 'T-MQ-1'")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(linked_sensor.as_deref(), Some("sensor-mq-1"));
}

#[tokio::test]
async fn handle_message_known_sensor_updates_only_status_and_publishes_event() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-known").await;
    sqlx::query!(
        r#"INSERT INTO trees (sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, watering_status)
        VALUES ('sensor-known', $1, 'Eiche', 'T-KN', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', 'unknown')"#,
        chrono::Utc::now().date_naive().format("%Y").to_string().parse::<i32>().unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    // 5 centibar < 25 lower → score=0 → Good.
    app.handle_mqtt_message(payload("sensor-known", 53.55, 9.99, 5))
        .await
        .unwrap();

    let tree_status: String = sqlx::query_scalar!(
        r#"SELECT watering_status::text AS "ws!" FROM trees WHERE number = 'T-KN'"#,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(tree_status, "good");
}
