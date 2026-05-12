use crate::helpers::spawn_app;
use serde_json::json;

fn valid_create_body(id: &str, model_id: i32) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "SN-001",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

#[tokio::test]
async fn create_sensor_returns_201_with_prepared_status() {
    let app = spawn_app().await;
    let resp = app
        .post_json("/api/v1/sensors", &valid_create_body("eui-001", 1))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "eui-001");
    assert_eq!(body["status"], "prepared");
    assert_eq!(body["sensor_type"], "lorawan");
    assert_eq!(body["model"]["id"], 1);
    assert_eq!(body["model"]["name"], "EcoDrizzler");
    assert!(body.get("coordinate").is_none_or(|c| c.is_null()));
    assert!(body.get("linked_tree_id").is_none_or(|c| c.is_null()));
}

#[tokio::test]
async fn create_sensor_with_unknown_model_returns_404() {
    let app = spawn_app().await;
    let resp = app
        .post_json("/api/v1/sensors", &valid_create_body("eui-002", 9999))
        .await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn create_sensor_with_duplicate_id_returns_409() {
    let app = spawn_app().await;
    let body = valid_create_body("eui-dup", 1);
    let first = app.post_json("/api/v1/sensors", &body).await;
    assert_eq!(first.status().as_u16(), 201);
    let second = app.post_json("/api/v1/sensors", &body).await;
    assert_eq!(second.status().as_u16(), 409);
}

#[tokio::test]
async fn create_sensor_with_invalid_hex_returns_400() {
    let app = spawn_app().await;
    let mut bad = valid_create_body("eui-bad", 1);
    bad["lorawan"]["dev_eui"] = json!("not-hex-not-hex");
    let resp = app.post_json("/api/v1/sensors", &bad).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn create_sensor_missing_lorawan_block_returns_400() {
    let app = spawn_app().await;
    let body = json!({
        "id": "eui-nolora",
        "sensor_type": "lorawan",
        "model_id": 1
    });
    let resp = app.post_json("/api/v1/sensors", &body).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn create_sensor_for_ges_1000_returns_201() {
    let app = spawn_app().await;
    let resp = app
        .post_json("/api/v1/sensors", &valid_create_body("eui-ges-1", 2))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["model"]["id"], 2);
    assert_eq!(body["model"]["name"], "GES-1000");
    assert_eq!(body["status"], "prepared");
}
