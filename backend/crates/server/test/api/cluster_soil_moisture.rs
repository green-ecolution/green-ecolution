use chrono::NaiveDate;
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

/// Tree + GES-1000 sensor wired straight into the DB; returns the sensor id.
async fn insert_sensor_tree(app: &TestApp, cluster_id: Uuid, sensor_id: &str) {
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
async fn day_buckets_aggregate_mean_min_max_per_depth() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-day").await;
    insert_reading(&app, "eui-sm-day", "2026-07-02 08:00:00", 40, 20.0).await;
    insert_reading(&app, "eui-sm-day", "2026-07-02 16:00:00", 40, 30.0).await;
    insert_reading(&app, "eui-sm-day", "2026-07-03 08:00:00", 40, 18.0).await;
    insert_reading(&app, "eui-sm-day", "2026-07-02 08:00:00", 80, 25.0).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();

    assert_eq!(body["bucket"], "day");
    let series = body["series"].as_array().unwrap();
    assert_eq!(series.len(), 2);
    let d40 = &series[0];
    assert_eq!(d40["depth_cm"], 40);
    let points = d40["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0]["mean"], 25.0);
    assert_eq!(points[0]["min"], 20.0);
    assert_eq!(points[0]["max"], 30.0);
    assert_eq!(points[0]["sample_count"], 2);
    assert_eq!(points[1]["mean"], 18.0);
    let d80 = &series[1];
    assert_eq!(d80["depth_cm"], 80);
    assert_eq!(d80["points"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn hour_bucket_splits_same_day_readings() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-hour").await;
    insert_reading(&app, "eui-sm-hour", "2026-07-02 08:10:00", 40, 20.0).await;
    insert_reading(&app, "eui-sm-hour", "2026-07-02 09:10:00", 40, 30.0).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}&bucket=hour"
        ))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    let points = body["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
}

#[tokio::test]
async fn sentinel_values_outside_percent_range_are_dropped() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-sentinel").await;
    insert_reading(&app, "eui-sm-sentinel", "2026-07-02 08:00:00", 40, 20.0).await;
    insert_reading(&app, "eui-sm-sentinel", "2026-07-02 09:00:00", 40, 6553.5).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    let points = body["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0]["sample_count"], 1);
    assert_eq!(points[0]["max"], 20.0);
}

#[tokio::test]
async fn thresholds_come_from_soil_condition() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-thresh").await;
    insert_reading(&app, "eui-sm-thresh", "2026-07-02 08:00:00", 40, 20.0).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}"
        ))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    // Uu @ 40 cm: nFK_eff = 20, PWP = 12 → moderate 20.0, critical 18.0.
    let t = body["thresholds"].as_array().unwrap();
    assert_eq!(t.len(), 1);
    assert_eq!(t[0]["depth_cm"], 40);
    assert_eq!(t[0]["moderate"], 20.0);
    assert_eq!(t[0]["critical"], 18.0);
}

#[tokio::test]
async fn unknown_soil_yields_empty_thresholds() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "unknown").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-unknown").await;
    insert_reading(&app, "eui-sm-unknown", "2026-07-02 08:00:00", 40, 20.0).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}"
        ))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["thresholds"].as_array().unwrap().len(), 0);
    assert_eq!(body["series"].as_array().unwrap().len(), 1);
    assert!(body["condition"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn cluster_without_sensor_data_returns_empty_series() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;

    let r = app
        .get(&format!("/api/v1/clusters/{cluster_id}/soil-moisture"))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["series"].as_array().unwrap().len(), 0);
    assert_eq!(body["thresholds"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn only_finished_watering_plans_appear_as_events() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    let finished_id = Uuid::now_v7();
    let planned_id = Uuid::now_v7();
    for (id, status, date) in [
        (finished_id, "finished", "2026-06-02"),
        (planned_id, "planned", "2026-07-15"),
    ] {
        sqlx::query(
            "INSERT INTO watering_plans (id, date, status, description)
             VALUES ($1, $2::date, $3::watering_plan_status, '')",
        )
        .bind(id)
        .bind(date.parse::<NaiveDate>().unwrap())
        .bind(status)
        .execute(&app.db_pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water)
             VALUES ($1, $2, 1800.0)",
        )
        .bind(cluster_id)
        .bind(id)
        .execute(&app.db_pool)
        .await
        .unwrap();
    }

    let r = app
        .get(&format!("/api/v1/clusters/{cluster_id}/soil-moisture"))
        .await;
    let body: serde_json::Value = r.json().await.unwrap();
    let events = body["watering_events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["watering_plan_id"], finished_id.to_string());
    assert_eq!(events[0]["date"], "2026-06-02");
    assert_eq!(events[0]["consumed_water_liters"], 1800.0);
}

#[tokio::test]
async fn missing_cluster_returns_404() {
    let app = spawn_app().await;
    let r = app
        .get(&format!(
            "/api/v1/clusters/{}/soil-moisture",
            Uuid::now_v7()
        ))
        .await;
    assert_eq!(r.status().as_u16(), 404);
}

#[tokio::test]
async fn invalid_bucket_and_range_return_400() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?bucket=week"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 400);

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?from=2026-07-10T00:00:00Z&to=2026-07-01T00:00:00Z"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 400);
}

#[tokio::test]
async fn condition_takes_worst_depth_and_maps_to_percent() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-cond").await;
    // Uu at both depths: PWP = 12, nFK_eff = 20.
    insert_reading(&app, "eui-sm-cond", "2026-07-02 08:00:00", 40, 25.0).await; // REW 65 %
    insert_reading(&app, "eui-sm-cond", "2026-07-02 08:00:00", 80, 15.0).await; // REW 15 %

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();

    let condition = body["condition"].as_array().unwrap();
    assert_eq!(condition.len(), 1);
    assert_eq!(condition[0]["worst_depth_cm"], 80);
    assert!((condition[0]["mean"].as_f64().unwrap() - 15.0).abs() < 1e-9);
    assert_eq!(body["condition_thresholds"]["moderate"], 40.0);
    assert_eq!(body["condition_thresholds"]["critical"], 30.0);
}

#[tokio::test]
async fn condition_is_clamped_to_percent_range() {
    let app = spawn_app().await;
    let cluster_id = create_cluster(&app, "Uu").await;
    insert_sensor_tree(&app, cluster_id, "eui-sm-clamp").await;
    // Uu @ 40 cm: PWP = 12, nFK_eff = 20 → REW(38.0) = 1.3, REW(5.0) = -0.35.
    insert_reading(&app, "eui-sm-clamp", "2026-07-02 08:00:00", 40, 38.0).await;
    insert_reading(&app, "eui-sm-clamp", "2026-07-03 08:00:00", 40, 5.0).await;

    let r = app
        .get(&format!(
            "/api/v1/clusters/{cluster_id}/soil-moisture?{WINDOW}&bucket=day"
        ))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();

    let condition = body["condition"].as_array().unwrap();
    assert_eq!(condition.len(), 2);
    assert_eq!(condition[0]["mean"], 100.0);
    assert_eq!(condition[0]["max"], 100.0);
    assert_eq!(condition[1]["mean"], 0.0);
    assert_eq!(condition[1]["min"], 0.0);
}
