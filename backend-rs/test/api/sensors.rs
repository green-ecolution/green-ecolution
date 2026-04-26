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
    assert_eq!(body["pagination"]["total"], 0);
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
    assert_eq!(body["pagination"]["total"], 2);
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
    assert_eq!(body["pagination"]["total"], 5);
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
