use crate::helpers::{TestApp, spawn_app};
use serde_json::json;

fn create_body(id: &str, model_id: i32) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "SN",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

async fn create_prepared_sensor(app: &TestApp, id: &str) {
    let r = app.post_json("/api/v1/sensors", &create_body(id, 1)).await;
    assert_eq!(
        r.status().as_u16(),
        201,
        "expected sensor create to succeed"
    );
}

async fn insert_tree(app: &TestApp, number: &str) -> i32 {
    sqlx::query_scalar!(
        r#"INSERT INTO trees (planting_year, species, number, latitude, longitude, geometry, description)
        VALUES (2020, 'Eiche', $1, $2, $3, ST_SetSRID(ST_MakePoint($3, $2), 4326), 'Test')
        RETURNING id"#,
        number,
        54.79_f64,
        9.45_f64,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn activate_happy_path_returns_200_with_coordinate_and_link() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-1").await;
    let tree_id = insert_tree(&app, "T-ACT-1").await;

    let r = app
        .post_json(
            "/api/v1/sensors/eui-act-1/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["status"], "offline");
    assert_eq!(body["linked_tree_id"], tree_id);
    assert!(body["coordinate"]["latitude"].as_f64().is_some());
    assert!(body["coordinate"]["longitude"].as_f64().is_some());
}

#[tokio::test]
async fn activate_idempotent_same_tree_returns_200() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-2").await;
    let tree_id = insert_tree(&app, "T-ACT-2").await;

    let r1 = app
        .post_json(
            "/api/v1/sensors/eui-act-2/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json(
            "/api/v1/sensors/eui-act-2/activate",
            &json!({ "tree_id": tree_id }),
        )
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
        .post_json(
            "/api/v1/sensors/eui-act-3/activate",
            &json!({ "tree_id": tree_a }),
        )
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json(
            "/api/v1/sensors/eui-act-3/activate",
            &json!({ "tree_id": tree_b }),
        )
        .await;
    assert_eq!(r2.status().as_u16(), 409);
}

#[tokio::test]
async fn activate_tree_already_has_sensor_returns_409() {
    let app = spawn_app().await;
    create_prepared_sensor(&app, "eui-act-4").await;
    create_prepared_sensor(&app, "eui-act-5").await;
    let tree_id = insert_tree(&app, "T-ACT-4").await;

    let r1 = app
        .post_json(
            "/api/v1/sensors/eui-act-4/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r1.status().as_u16(), 200);

    let r2 = app
        .post_json(
            "/api/v1/sensors/eui-act-5/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r2.status().as_u16(), 409);
}

#[tokio::test]
async fn activate_unknown_sensor_returns_404() {
    let app = spawn_app().await;
    let tree_id = insert_tree(&app, "T-ACT-NS").await;

    let r = app
        .post_json(
            "/api/v1/sensors/eui-nope/activate",
            &json!({ "tree_id": tree_id }),
        )
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
            &json!({ "tree_id": 999_999 }),
        )
        .await;
    assert_eq!(r.status().as_u16(), 404);
}
