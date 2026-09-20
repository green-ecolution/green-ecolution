use crate::helpers::spawn_app;

async fn insert_sensor(app: &crate::helpers::TestApp, id: &str) {
    let model_id = app.ecodrizzler_model_id().await;
    insert_sensor_with_model(app, id, model_id).await;
}

async fn insert_sensor_with_model(app: &crate::helpers::TestApp, id: &str, model_id: uuid::Uuid) {
    sqlx::query!(
        r#"INSERT INTO sensors (id, activated_at, type, model_id, organization_id)
        VALUES ($1, NOW(), 'lorawan', $2, '01980000-0000-7000-8000-000000000001')"#,
        id,
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

// Raw sqlx::query so this fixture doesn't need an offline-cache entry.
async fn link_sensor_to_new_tree(app: &crate::helpers::TestApp, sensor_id: &str) {
    sqlx::query(
        r#"INSERT INTO trees (id, sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, organization_id)
        VALUES ($1, $2, 2020, 'Eiche', $3, 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test',
                '01980000-0000-7000-8000-000000000001')"#,
    )
    .bind(uuid::Uuid::now_v7())
    .bind(sensor_id)
    .bind(format!("T-{sensor_id}"))
    .execute(&app.db_pool)
    .await
    .unwrap();
}

// Raw sqlx::query so this fixture doesn't need an offline-cache entry.
async fn link_sensor_to_new_tree_in_cluster(
    app: &crate::helpers::TestApp,
    sensor_id: &str,
    cluster_name: &str,
) -> uuid::Uuid {
    let cluster_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO tree_clusters (id, name, address, description, moisture_level,
                                      organization_id)
        VALUES ($1, $2, 'Teststraße 1', 'Test', 0.5,
                '01980000-0000-7000-8000-000000000001')"#,
    )
    .bind(cluster_id)
    .bind(cluster_name)
    .execute(&app.db_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO trees (id, sensor_id, tree_cluster_id, planting_year, species, number,
                              latitude, longitude, geometry, description, organization_id)
        VALUES ($1, $2, $3, 2020, 'Eiche', $4, 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test',
                '01980000-0000-7000-8000-000000000001')"#,
    )
    .bind(uuid::Uuid::now_v7())
    .bind(sensor_id)
    .bind(cluster_id)
    .bind(format!("T-{sensor_id}"))
    .execute(&app.db_pool)
    .await
    .unwrap();

    cluster_id
}

async fn list_ids(app: &crate::helpers::TestApp, query: &str) -> Vec<String> {
    let body: serde_json::Value = app
        .get(&format!("/api/v1/sensors?{query}"))
        .await
        .json()
        .await
        .unwrap();
    body["data"]
        .as_array()
        .expect("list response carries a data array")
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn insert_reading(app: &crate::helpers::TestApp, sensor_id: &str, data: serde_json::Value) {
    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, $3)"#,
        uuid::Uuid::now_v7(),
        sensor_id,
        data,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_reading_at(
    app: &crate::helpers::TestApp,
    sensor_id: &str,
    data: serde_json::Value,
    updated_at: chrono::NaiveDateTime,
) {
    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data, updated_at) VALUES ($1, $2, $3, $4)"#,
        uuid::Uuid::now_v7(),
        sensor_id,
        data,
        updated_at,
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

    let model_id = app.ecodrizzler_model_id().await;
    insert_sensor(&app, "sensor-100").await;

    let response = app.get("/api/v1/sensors/sensor-100").await;

    assert_eq!(response.status().as_u16(), 200);

    let sensor: serde_json::Value = response.json().await.unwrap();
    assert_eq!(sensor["id"], "sensor-100");
    assert_eq!(sensor["status"], "offline");
    assert_eq!(sensor["sensor_type"], "lorawan");
    assert_eq!(sensor["model"]["id"], model_id.to_string());
    let abilities = sensor["model"]["abilities"].as_array().unwrap();
    assert_eq!(abilities.len(), 6);
    let tension_depths: Vec<i64> = abilities
        .iter()
        .filter(|a| a["ability"] == "soil_tension")
        .map(|a| a["depth_cm"].as_i64().unwrap())
        .collect();
    assert_eq!(tension_depths, vec![30, 60, 90]);
    assert!(
        abilities.iter().any(|a| a["ability"] == "soil_moisture"
            && a["depth_cm"] == 15
            && a["unit"] == "percent")
    );
    // No tree linked → coordinate / linked_tree_id are omitted.
    assert!(sensor.get("coordinate").is_none_or(|c| c.is_null()));
    assert!(sensor.get("linked_tree_id").is_none_or(|c| c.is_null()));
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

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn list_sensor_data_returns_inserted_data() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-data").await;

    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, $3)"#,
        uuid::Uuid::now_v7(),
        "sensor-data",
        serde_json::json!({"temperature": 22.5})
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.get("/api/v1/sensors/sensor-data/data").await;

    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["data"]["temperature"], 22.5);
    assert_eq!(body["pagination"]["total_records"], 1);
}

#[tokio::test]
async fn list_sensor_data_respects_pagination() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-001").await;
    for i in 0..3 {
        insert_reading(
            &app,
            "sensor-001",
            serde_json::json!({ "battery": 3.6, "n": i }),
        )
        .await;
    }

    let response = app
        .get("/api/v1/sensors/sensor-001/data?page=1&per_page=2")
        .await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 3);
    assert_eq!(body["pagination"]["total_pages"], 2);
    assert_eq!(body["pagination"]["next_page"], 2);

    let response = app
        .get("/api/v1/sensors/sensor-001/data?page=2&per_page=2")
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_sensor_data_filters_by_time_range() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-001").await;
    let old = chrono::NaiveDate::from_ymd_opt(2026, 6, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let recent = chrono::NaiveDate::from_ymd_opt(2026, 7, 10)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    insert_reading_at(
        &app,
        "sensor-001",
        serde_json::json!({ "battery": 3.2 }),
        old,
    )
    .await;
    insert_reading_at(
        &app,
        "sensor-001",
        serde_json::json!({ "battery": 3.6 }),
        recent,
    )
    .await;

    let response = app
        .get("/api/v1/sensors/sensor-001/data?from=2026-07-01T00:00:00Z")
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["data"]["battery"], 3.6);
    assert_eq!(body["pagination"]["total_records"], 1);

    let response = app
        .get("/api/v1/sensors/sensor-001/data?to=2026-07-01T00:00:00Z")
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["data"]["battery"], 3.2);
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
        r#"INSERT INTO trees (id, sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, organization_id)
        VALUES ($1, 'sensor-unlink', 2020, 'Eiche', 'T-UNL', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', '01980000-0000-7000-8000-000000000001')"#,
        uuid::Uuid::now_v7(),
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
        r#"INSERT INTO trees (id, sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, organization_id)
        VALUES ($1, 'sensor-tree', 2020, 'Eiche', 'T-LINK', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', '01980000-0000-7000-8000-000000000001')"#,
        uuid::Uuid::now_v7(),
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

#[tokio::test]
async fn ingest_via_create_and_activate_updates_watering_status() {
    let app = spawn_app().await;

    // 1. Register a prepared sensor through the public API.
    let model_id = app.ecodrizzler_model_id().await;
    let create_body = serde_json::json!({
        "id": "sensor-mq-1",
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "SN", "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1", "app_key": "00112233445566778899aabbccddeeff"
        }
    });
    let r = app.post_json("/api/v1/sensors", &create_body).await;
    assert_eq!(r.status().as_u16(), 201);

    // 2. Insert a tree (planted this year so year=0 calibration applies)
    //    and activate the sensor against it.
    let planting_year: i32 = chrono::Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap();
    let tree_id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description, organization_id)
        VALUES ($1, $2, 'Eiche', 'T-MQ-1', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', '01980000-0000-7000-8000-000000000001')"#,
        tree_id,
        planting_year,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let act = app
        .post_json(
            "/api/v1/sensors/sensor-mq-1/activate",
            &serde_json::json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(act.status().as_u16(), 200);

    // 3. 50 centibar with 0/1-year defaults (lower=25, higher=33) → score=2 → Bad.
    app.ingest_ecodrizzler("sensor-mq-1", 50)
        .await
        .expect("ingest should succeed");

    let tree_status: String = sqlx::query_scalar!(
        r#"SELECT watering_status::text AS "ws!" FROM trees WHERE number = 'T-MQ-1'"#,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(tree_status, "bad");

    let linked_sensor: Option<String> =
        sqlx::query_scalar!("SELECT sensor_id FROM trees WHERE number = 'T-MQ-1'")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(linked_sensor.as_deref(), Some("sensor-mq-1"));
}

#[tokio::test]
async fn list_sensors_includes_latest_reading() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-latest").await;
    insert_reading(
        &app,
        "sensor-latest",
        serde_json::json!({"temperature": 22.5}),
    )
    .await;

    let response = app.get("/api/v1/sensors").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let sensor = &body["data"][0];
    assert_eq!(sensor["id"], "sensor-latest");

    let latest = &sensor["latest_data"];
    assert!(
        !latest.is_null(),
        "latest_data should be present in the list response, got: {sensor}"
    );
    assert_eq!(latest["data"]["temperature"], 22.5);
    assert!(latest["id"].is_string());
    assert!(latest["created_at"].is_string());
    assert!(latest["updated_at"].is_string());
}

#[tokio::test]
async fn list_trees_embeds_sensor_latest_reading() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-tree-data").await;
    sqlx::query!(
        r#"INSERT INTO trees (id, sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, organization_id)
        VALUES ($1, 'sensor-tree-data', 2020, 'Eiche', 'T-DATA', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', '01980000-0000-7000-8000-000000000001')"#,
        uuid::Uuid::now_v7(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    insert_reading(
        &app,
        "sensor-tree-data",
        serde_json::json!({"temperature": 19.1}),
    )
    .await;

    // The tree list endpoint batch-resolves embedded sensors via view_by_ids.
    let response = app.get("/api/v1/trees").await;
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    let tree = &body["data"][0];
    let latest = &tree["sensor"]["latest_data"];
    assert!(
        !latest.is_null(),
        "latest_data should be embedded via view_by_ids, got: {}",
        tree["sensor"]
    );
    assert_eq!(latest["data"]["temperature"], 19.1);
}

#[tokio::test]
async fn ingest_for_known_sensor_updates_tree_watering_status() {
    let app = spawn_app().await;

    insert_sensor(&app, "sensor-known").await;
    sqlx::query!(
        r#"INSERT INTO trees (id, sensor_id, planting_year, species, number, latitude, longitude,
                              geometry, description, watering_status, organization_id)
        VALUES ($1, 'sensor-known', $2, 'Eiche', 'T-KN', 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', 'unknown', '01980000-0000-7000-8000-000000000001')"#,
        uuid::Uuid::now_v7(),
        chrono::Utc::now()
            .date_naive()
            .format("%Y")
            .to_string()
            .parse::<i32>()
            .unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    // 5 centibar < 25 lower → score=0 → Good.
    app.ingest_ecodrizzler("sensor-known", 5).await.unwrap();

    let tree_status: String = sqlx::query_scalar!(
        r#"SELECT watering_status::text AS "ws!" FROM trees WHERE number = 'T-KN'"#,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(tree_status, "good");
}

#[tokio::test]
async fn sensor_list_searches_eui_model_name_and_cluster_name() {
    let app = spawn_app().await;
    let eco = app.ecodrizzler_model_id().await;
    let ges = app.ges_1000_model_id().await;
    insert_sensor_with_model(&app, "eui-a81758fffe0c3b52", eco).await;
    insert_sensor_with_model(&app, "eui-b91869fffe1d4c63", ges).await;
    link_sensor_to_new_tree_in_cluster(&app, "eui-a81758fffe0c3b52", "Hafenspitze").await;

    // Matches neither EUI, only the `GES-1000` model name.
    assert_eq!(
        list_ids(&app, "q=ges").await,
        vec!["eui-b91869fffe1d4c63".to_owned()]
    );
    assert_eq!(
        list_ids(&app, "q=A81758").await,
        vec!["eui-a81758fffe0c3b52".to_owned()]
    );
    // Matches neither EUI nor model name, only the linked tree's cluster.
    assert_eq!(
        list_ids(&app, "q=hafen").await,
        vec!["eui-a81758fffe0c3b52".to_owned()]
    );
}

#[tokio::test]
async fn sensor_list_filters_by_linked_tree() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-000000000000000a").await;
    insert_sensor(&app, "eui-000000000000000b").await;
    link_sensor_to_new_tree(&app, "eui-000000000000000a").await;

    assert_eq!(
        list_ids(&app, "has_tree=true").await,
        vec!["eui-000000000000000a".to_owned()]
    );
    assert_eq!(
        list_ids(&app, "has_tree=false").await,
        vec!["eui-000000000000000b".to_owned()]
    );
}

#[tokio::test]
async fn sensor_list_filters_by_cluster_of_the_linked_tree() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-000000000000000a").await;
    insert_sensor(&app, "eui-000000000000000b").await;
    insert_sensor(&app, "eui-000000000000000c").await;
    let north = link_sensor_to_new_tree_in_cluster(&app, "eui-000000000000000a", "Nord").await;
    let south = link_sensor_to_new_tree_in_cluster(&app, "eui-000000000000000b", "Süd").await;
    // Linked to a tree, but that tree sits in no cluster.
    link_sensor_to_new_tree(&app, "eui-000000000000000c").await;

    assert_eq!(
        list_ids(&app, &format!("cluster_id={north}")).await,
        vec!["eui-000000000000000a".to_owned()]
    );
    assert_eq!(
        list_ids(&app, &format!("cluster_id={north}&cluster_id={south}")).await,
        vec![
            "eui-000000000000000a".to_owned(),
            "eui-000000000000000b".to_owned()
        ]
    );
}

#[tokio::test]
async fn sensor_list_exposes_the_cluster_of_the_linked_tree() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-000000000000000a").await;
    insert_sensor(&app, "eui-000000000000000b").await;
    let cluster_id = link_sensor_to_new_tree_in_cluster(&app, "eui-000000000000000a", "Nord").await;
    link_sensor_to_new_tree(&app, "eui-000000000000000b").await;

    let body: serde_json::Value = app.get("/api/v1/sensors").await.json().await.unwrap();
    let sensors = body["data"].as_array().unwrap();

    let linked = &sensors[0];
    assert_eq!(linked["id"], "eui-000000000000000a");
    assert_eq!(linked["linked_cluster_id"], cluster_id.to_string());
    assert_eq!(linked["linked_cluster_name"], "Nord");

    // A tree without a cluster leaves both fields off the payload.
    let clusterless = &sensors[1];
    assert_eq!(clusterless["id"], "eui-000000000000000b");
    assert!(clusterless.get("linked_cluster_id").is_none());
    assert!(clusterless.get("linked_cluster_name").is_none());
}

#[tokio::test]
async fn sensor_list_model_parameter_is_repeatable() {
    let app = spawn_app().await;
    let eco = app.ecodrizzler_model_id().await;
    let ges = app.ges_1000_model_id().await;
    insert_sensor_with_model(&app, "eui-000000000000000a", eco).await;
    insert_sensor_with_model(&app, "eui-000000000000000b", ges).await;

    assert_eq!(
        list_ids(&app, &format!("model_id={eco}")).await,
        vec!["eui-000000000000000a".to_owned()]
    );
    assert_eq!(
        list_ids(&app, &format!("model_id={eco}&model_id={ges}"))
            .await
            .len(),
        2
    );
}

#[tokio::test]
async fn sensor_list_sorts_by_last_reading_descending() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-000000000000000a").await;
    insert_sensor(&app, "eui-000000000000000b").await;
    insert_reading(&app, "eui-000000000000000a", serde_json::json!({"n": 1})).await;
    insert_reading(&app, "eui-000000000000000b", serde_json::json!({"n": 2})).await;

    let ids = list_ids(&app, "sort=last_reading&order=desc").await;

    assert_eq!(
        ids.first().map(String::as_str),
        Some("eui-000000000000000b")
    );
}

#[tokio::test]
async fn sensor_list_reports_the_prefilter_total() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-000000000000000a").await;
    insert_sensor(&app, "eui-000000000000000b").await;

    let body: serde_json::Value = app
        .get("/api/v1/sensors?q=000000000000000a")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["pagination"]["total_records"], 1);
    assert_eq!(body["pagination"]["total_unfiltered"], 2);
}

#[tokio::test]
async fn sensor_list_rejects_an_overlong_search_term() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/sensors?q={}", "x".repeat(101)))
        .await;

    assert_eq!(response.status().as_u16(), 400);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"].is_string());
}
