use serde_json::json;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::spawn_app;

const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";

fn tree_payload(org: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "species": "Quercus robur", "number": "SCOPE-001", "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": ""
    });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

#[tokio::test]
async fn create_tree_stores_explicit_organization() {
    let app = spawn_app().await;
    // demo bypass: unrestricted, explicit org wins
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, '01980000-0000-7000-8000-000000000001', 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let resp = app
        .post_json("/api/v1/trees", &tree_payload(Some(TBZ_ORG)))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}

#[tokio::test]
async fn create_tree_defaults_to_root_in_demo_mode() {
    let app = spawn_app().await;
    let resp = app.post_json("/api/v1/trees", &tree_payload(None)).await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["organization_id"],
        "01980000-0000-7000-8000-000000000001"
    );
}

#[tokio::test]
async fn create_tree_requires_create_permission_in_target_org() {
    let (harness, app) = spawn_with_auth().await;
    // user without any grants
    let token = harness.sign_token(json!({ "sub": uuid::Uuid::now_v7().to_string() }));
    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token)
        .json(&tree_payload(Some("01980000-0000-7000-8000-000000000001")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
