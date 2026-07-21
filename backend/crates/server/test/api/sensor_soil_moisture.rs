use uuid::Uuid;

use crate::helpers::{TestApp, spawn_app};

async fn create_cluster(app: &TestApp, soil: &str) -> Uuid {
    let body = serde_json::json!({
        "name": "Hafenspitze",
        "address": "Schiffbrücke 12",
        "description": "Testgruppe",
        "soil_condition": soil,
        "tree_ids": []
    });
    let r = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(r.status().as_u16(), 201, "cluster create failed");
    let v: serde_json::Value = r.json().await.unwrap();
    v["id"].as_str().unwrap().parse().unwrap()
}

/// GES-1000 sensor wired straight into the DB, optionally linked to a tree
/// in `cluster_id`.
async fn insert_sensor(app: &TestApp, cluster_id: Option<Uuid>, sensor_id: &str) {
    let model_id = app.ges_1000_model_id().await;
    sqlx::query!(
        r#"INSERT INTO sensors (id, activated_at, type, model_id) VALUES ($1, NOW(), 'lorawan', $2)"#,
        sensor_id,
        model_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
           VALUES ($1, '', '', '', '')"#,
        sensor_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    if let Some(cluster_id) = cluster_id {
        sqlx::query!(
            r#"INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number,
                              latitude, longitude, geometry)
           VALUES ($1, $2, $3, 2020, 'Stieleiche', $4, 54.79, 9.43,
                   ST_SetSRID(ST_MakePoint(9.43, 54.79), 4326))"#,
            Uuid::now_v7(),
            cluster_id,
            sensor_id,
            format!("T-{sensor_id}"),
        )
        .execute(&app.db_pool)
        .await
        .unwrap();
    }
}

async fn soil_moisture_ability_id(app: &TestApp, depth_cm: i32) -> Uuid {
    sqlx::query_scalar!(
        r#"SELECT sma.id
           FROM sensor_model_abilities sma
           JOIN sensor_models m ON m.id = sma.sensor_model_id AND m.name = 'GES-1000'
           JOIN sensor_abilities sa ON sa.id = sma.sensor_ability_id
           WHERE sa.ability = 'soil_moisture' AND sma.depth_cm = $1"#,
        depth_cm,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

/// One reading with an explicit wall-clock time (the ingest path always
/// stamps now(), which makes bucket assertions impossible).
async fn insert_reading(app: &TestApp, sensor_id: &str, at: &str, depth_cm: i32, value: f64) {
    let data_id = Uuid::now_v7();
    let ability_id = soil_moisture_ability_id(app, depth_cm).await;
    sqlx::query!(
        r#"INSERT INTO sensor_data (id, sensor_id, data, updated_at)
           VALUES ($1, $2, '{}'::jsonb, $3::timestamp)"#,
        data_id,
        sensor_id,
        chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M:%S").unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO sensor_data_ability_values (sensor_data_id, sensor_model_ability_id, value)
           VALUES ($1, $2, $3::float8::numeric)"#,
        data_id,
        ability_id,
        value,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

const WINDOW: &str = "from=2026-07-01T00:00:00Z&to=2026-07-10T00:00:00Z";

#[tokio::test]
async fn linked_sensor_gets_series_thresholds_and_condition() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor(&app, Some(cluster_id), "eui-ssm-linked").await;
    // Uu at both depths: PWP = 12, nFK_eff = 20.
    insert_reading(&app, "eui-ssm-linked", "2026-07-02 08:00:00", 40, 25.0).await; // REW 65 %
    insert_reading(&app, "eui-ssm-linked", "2026-07-02 08:00:00", 80, 15.0).await; // REW 15 %

    let r = app
        .get(&format!(
            "/api/v1/sensors/eui-ssm-linked/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();

    assert_eq!(body["series"].as_array().unwrap().len(), 2);
    assert_eq!(body["thresholds"].as_array().unwrap().len(), 2);
    let condition = body["condition"].as_array().unwrap();
    assert_eq!(condition.len(), 1);
    assert_eq!(condition[0]["worst_depth_cm"], 80);
    assert!((condition[0]["mean"].as_f64().unwrap() - 15.0).abs() < 1e-9);
}

#[tokio::test]
async fn series_only_contains_the_requested_sensor() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor(&app, Some(cluster_id), "eui-ssm-a").await;
    insert_sensor(&app, Some(cluster_id), "eui-ssm-b").await;
    insert_reading(&app, "eui-ssm-a", "2026-07-02 08:00:00", 40, 20.0).await;
    insert_reading(&app, "eui-ssm-b", "2026-07-02 08:00:00", 40, 90.0).await;

    let r = app
        .get(&format!(
            "/api/v1/sensors/eui-ssm-a/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    let points = body["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0]["mean"], 20.0);
    assert_eq!(points[0]["sample_count"], 1);
}

#[tokio::test]
async fn unlinked_sensor_gets_series_without_thresholds_or_events() {
    let app = spawn_app().await;
    insert_sensor(&app, None, "eui-ssm-lone").await;
    insert_reading(&app, "eui-ssm-lone", "2026-07-02 08:00:00", 40, 20.0).await;

    let r = app
        .get(&format!(
            "/api/v1/sensors/eui-ssm-lone/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["series"].as_array().unwrap().len(), 1);
    assert!(body["thresholds"].as_array().unwrap().is_empty());
    assert!(body["condition"].as_array().unwrap().is_empty());
    assert!(body["watering_events"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn unknown_sensor_is_404() {
    let app = spawn_app().await;
    let r = app
        .get(&format!(
            "/api/v1/sensors/eui-does-not-exist/soil-moisture?{WINDOW}"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 404);
}
