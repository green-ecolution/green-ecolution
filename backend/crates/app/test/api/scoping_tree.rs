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

fn numbered_tree_payload(number: &str, org: Option<&str>) -> serde_json::Value {
    let mut p = tree_payload(org);
    p["number"] = json!(number);
    p
}

async fn insert_tbz_org(app: &crate::helpers::TestApp) {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, '01980000-0000-7000-8000-000000000001', 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn create_tree_stores_explicit_organization() {
    let app = spawn_app().await;
    // demo bypass: unrestricted, explicit org wins
    insert_tbz_org(&app).await;
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

#[tokio::test]
async fn tree_markers_can_be_filtered_by_organization() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;

    let resp = app
        .post_json("/api/v1/trees", &numbered_tree_payload("SCOPE-M-001", None))
        .await;
    assert_eq!(resp.status(), 201);
    let resp = app
        .post_json(
            "/api/v1/trees",
            &numbered_tree_payload("SCOPE-M-002", Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);

    let resp = app
        .get(&format!(
            "/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&organization_id={TBZ_ORG}"
        ))
        .await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "SCOPE-M-002");
}

#[tokio::test]
async fn tree_markers_carry_their_organization() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;

    let resp = app
        .post_json(
            "/api/v1/trees",
            &numbered_tree_payload("SCOPE-M-003", Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);

    // Unfiltered: the cluster forms need the org per marker to dim foreign trees.
    let resp = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46")
        .await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["organization_id"], TBZ_ORG);
}
