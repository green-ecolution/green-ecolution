use crate::{
    helpers::{TestApp, spawn_app},
    sensors::{create_prepared_sensor, tree_target},
};
use uuid::Uuid;

async fn insert_tree(app: &TestApp, number: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description, organization_id)
        VALUES ($1, 2020, 'Eiche', $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), 'Test', '01980000-0000-7000-8000-000000000001')"#,
        id,
        number,
        54.79_f64,
        9.45_f64,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn activate_happy_path_returns_200_with_location_and_target() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-1").await;
    let tree_id = insert_tree(&app, "T-ACT-1").await;

    let r = app
        .post_json("/api/v1/sensors/eui-act-1/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "offline");
    assert_eq!(body["target"]["id"], tree_id.to_string());
    assert!(body["target_label"].is_null());
    assert!(body["location"]["latitude"].as_f64().is_some());
    assert!(body["location"]["longitude"].as_f64().is_some());
}

#[tokio::test]
async fn activate_idempotent_same_tree_returns_200() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-2").await;
    let tree_id = insert_tree(&app, "T-ACT-2").await;

    let r1 = app
        .post_json("/api/v1/sensors/eui-act-2/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json("/api/v1/sensors/eui-act-2/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r2.status().as_u16(), 200);
}

#[tokio::test]
async fn activate_already_active_other_tree_returns_409() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-3").await;
    let tree_a = insert_tree(&app, "T-ACT-3A").await;
    let tree_b = insert_tree(&app, "T-ACT-3B").await;

    let r1 = app
        .post_json("/api/v1/sensors/eui-act-3/activate", &tree_target(tree_a))
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json("/api/v1/sensors/eui-act-3/activate", &tree_target(tree_b))
        .await;
    assert_eq!(r2.status().as_u16(), 409);
    let body: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(body["code"], "conflict.sensor_already_activated");
}

#[tokio::test]
async fn activate_tree_already_has_sensor_returns_409() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-4").await;
    create_prepared_sensor(&app, "eui-act-5").await;
    let tree_id = insert_tree(&app, "T-ACT-4").await;

    let r1 = app
        .post_json("/api/v1/sensors/eui-act-4/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json("/api/v1/sensors/eui-act-5/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r2.status().as_u16(), 409);
    let body: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(body["code"], "conflict.target_already_has_sensor");
}

#[tokio::test]
async fn activate_unknown_sensor_returns_404() {
    let app = spawn_app().await;
    let tree_id = insert_tree(&app, "T-ACT-NS").await;

    let r = app
        .post_json("/api/v1/sensors/eui-nope/activate", &tree_target(tree_id))
        .await;
    assert_eq!(r.status().as_u16(), 404);
}

#[tokio::test]
async fn activate_unknown_tree_returns_404() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-6").await;

    let r = app
        .post_json(
            "/api/v1/sensors/eui-act-6/activate",
            &tree_target(Uuid::now_v7()),
        )
        .await;
    assert_eq!(r.status().as_u16(), 404);
}
