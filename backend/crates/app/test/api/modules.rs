use serde_json::json;
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

#[tokio::test]
async fn list_holds_the_registered_modules() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/modules").await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["modules"], json!(["irrigation", "waste"]));
    assert!(
        body["organizations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["organization_id"] == ROOT_ORG_ID && o["active"] == json!(["irrigation"]))
    );
}

#[tokio::test]
async fn switching_an_unknown_module_is_rejected() {
    let app = spawn_app().await;
    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{ROOT_ORG_ID}/modules"),
            &json!({ "modules": { "nope": true } }),
        )
        .await;
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.unknown");
}

#[tokio::test]
async fn unknown_organization_is_not_found() {
    let app = spawn_app().await;
    let resp = app
        .get(&format!("/api/v1/organizations/{}/modules", Uuid::now_v7()))
        .await;
    assert_eq!(resp.status(), 404);
}
