use crate::{
    helpers::{TestApp, spawn_app},
    sensors::{create_prepared_sensor, tree_target},
};
use uuid::Uuid;

pub async fn insert_tree(app: &TestApp, number: &str) -> Uuid {
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

async fn activate(app: &TestApp, sensor_id: &str, tree_id: Uuid) {
    let r = app
        .post_json(
            &format!("/api/v1/sensors/{sensor_id}/activate"),
            &tree_target(tree_id),
        )
        .await;
    assert_eq!(r.status().as_u16(), 200, "expected activate to succeed");
}

#[tokio::test]
async fn reassign_moves_sensor_to_new_tree() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-rel-1").await;
    let tree_a = insert_tree(&app, "T-REL-A").await;
    let tree_b = insert_tree(&app, "T-REL-B").await;
    activate(&app, "eui-rel-1", tree_a).await;

    let r = app
        .put_json("/api/v1/sensors/eui-rel-1/target", &tree_target(tree_b))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["target"]["id"], tree_b.to_string());

    let tree = app.get("/api/v1/sensors/eui-rel-1/tree").await;
    assert_eq!(tree.status().as_u16(), 200);
    let tree_body: serde_json::Value = tree.json().await.unwrap();
    assert_eq!(tree_body["id"], tree_b.to_string());
}

#[tokio::test]
async fn reassign_to_tree_with_other_sensor_conflicts() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-rel-2a").await;
    create_prepared_sensor(&app, "eui-rel-2b").await;
    let tree_a = insert_tree(&app, "T-REL-2A").await;
    let tree_b = insert_tree(&app, "T-REL-2B").await;
    activate(&app, "eui-rel-2a", tree_a).await;
    activate(&app, "eui-rel-2b", tree_b).await;

    let r = app
        .put_json("/api/v1/sensors/eui-rel-2a/target", &tree_target(tree_b))
        .await;
    assert_eq!(r.status().as_u16(), 409);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["code"], "conflict.target_already_has_sensor");
}

#[tokio::test]
async fn reassign_on_prepared_sensor_conflicts() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-rel-5").await;
    let tree = insert_tree(&app, "T-REL-5").await;

    let r = app
        .put_json("/api/v1/sensors/eui-rel-5/target", &tree_target(tree))
        .await;
    assert_eq!(r.status().as_u16(), 409);
}

#[tokio::test]
async fn remove_target_resets_sensor_to_prepared() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-rel-3").await;
    let tree = insert_tree(&app, "T-REL-3").await;
    activate(&app, "eui-rel-3", tree).await;

    let r = app.delete("/api/v1/sensors/eui-rel-3/target").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "prepared");
    assert!(body["target"].is_null());

    let tree_after = app.get("/api/v1/sensors/eui-rel-3/tree").await;
    assert_eq!(tree_after.status().as_u16(), 404);
}

#[tokio::test]
async fn remove_target_on_prepared_sensor_is_idempotent() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-rel-4").await;

    let r = app.delete("/api/v1/sensors/eui-rel-4/target").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "prepared");
}
