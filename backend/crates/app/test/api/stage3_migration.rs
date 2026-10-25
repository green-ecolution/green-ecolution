use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::helpers::{BEFORE_STAGE_3, TestApp, spawn_app_on_migrated_until};

const ROOT: &str = "01980000-0000-7000-8000-000000000001";
const SENSOR: &str = "eui-migration-1";
const CLUSTER: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0c01);
const TREE: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0e01);
const FINISHED: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0f01);
const PLANNED: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0f02);
const TRANSPORTER: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0a01);

async fn seed_old_schema(pool: &PgPool) {
    sqlx::raw_sql(include_str!("stage3_migration_seed.sql"))
        .execute(pool)
        .await
        .expect("seed old schema");
}

async fn get_json(app: &TestApp, path: &str) -> Value {
    let response = app.get(path).await;
    assert_eq!(response.status(), 200, "GET {path}");
    response.json().await.unwrap()
}

#[tokio::test]
async fn the_irrigation_stock_survives_stage_3() {
    let app = spawn_app_on_migrated_until(BEFORE_STAGE_3, async |pool: &PgPool| {
        seed_old_schema(pool).await
    })
    .await;

    let settings = get_json(
        &app,
        &format!("/api/v1/organizations/{ROOT}/settings/irrigation"),
    )
    .await;
    assert_eq!(settings["water_demand"]["value"], 55.0);
    assert_eq!(settings["just_watered_ttl_secs"]["value"], 7200);

    let sensor = get_json(&app, &format!("/api/v1/sensors/{SENSOR}")).await;
    assert_eq!(sensor["target"]["module"], "irrigation");
    assert_eq!(sensor["target"]["kind"], "tree");
    assert_eq!(sensor["target"]["id"], TREE.to_string());

    let tree = get_json(&app, &format!("/api/v1/trees/{TREE}")).await;
    assert_eq!(tree["sensor"]["id"], SENSOR);

    let finished = get_json(&app, &format!("/api/v1/plans/{FINISHED}")).await;
    assert_eq!(finished["module"], "irrigation");
    assert_eq!(finished["stops"], json!([CLUSTER]));
    assert_eq!(finished["outcomes"][0]["completed"], true);
    assert_eq!(finished["outcomes"][0]["amount"], 140.0);

    let planned = get_json(&app, &format!("/api/v1/plans/{PLANNED}")).await;
    assert_eq!(planned["outcomes"][0]["completed"], false);
    assert!(planned["outcomes"][0]["amount"].is_null());

    let vehicle = get_json(&app, &format!("/api/v1/vehicles/{TRANSPORTER}")).await;
    assert_eq!(vehicle["capacities"]["water"], 3000.0);

    let start_points = get_json(&app, "/api/v1/routing/start-points").await;
    assert!(
        start_points
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["replenishes"] == json!(["water"]))
    );

    let cluster_comments = get_json(&app, &format!("/api/v1/clusters/{CLUSTER}/comments")).await;
    assert_eq!(cluster_comments["data"].as_array().unwrap().len(), 1);
    let plan_comments = get_json(&app, &format!("/api/v1/plans/{FINISHED}/comments")).await;
    assert_eq!(plan_comments["data"].as_array().unwrap().len(), 1);
}
