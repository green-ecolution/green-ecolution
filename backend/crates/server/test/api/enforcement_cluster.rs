use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::{TestApp, seed_user_with_permissions};

fn cluster_payload(org: Uuid) -> serde_json::Value {
    json!({
        "name": "Enforcement Cluster",
        "address": "Am Testfeld 1",
        "description": "",
        "soil_condition": "Su3",
        "tree_ids": [],
        "organization_id": org,
    })
}

#[tokio::test]
async fn foreign_org_sees_no_clusters_and_gets_404_on_detail() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) = seed_user_with_permissions(
        &harness,
        &app,
        "Org A",
        &["tree_cluster:read", "tree_cluster:create"],
    )
    .await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["tree_cluster:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&token_a)
        .json(&cluster_payload(org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cluster_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b["data"].as_array().unwrap().len(), 0);

    let detail_b = client
        .get(format!("{}/api/v1/clusters/{cluster_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_a["data"].as_array().unwrap().len(), 1);
}

/// Seeds a read-only role in an already-existing org (used to test that
/// visibility and update permission are checked separately).
async fn seed_reader_in_org(harness: &AuthHarness, app: &TestApp, org: Uuid) -> (Uuid, String) {
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Reader', ARRAY['tree_cluster:read'])
           RETURNING id"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)"#,
        user_id,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    (
        user_id,
        harness.sign_token(json!({ "sub": user_id.to_string() })),
    )
}

#[tokio::test]
async fn visible_without_update_permission_yields_403() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token_admin) = seed_user_with_permissions(
        &harness,
        &app,
        "Org",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let (_, token_reader) = seed_reader_in_org(&harness, &app, org).await;

    let client = reqwest::Client::new();
    let created: serde_json::Value = client
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&token_admin)
        .json(&cluster_payload(org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cluster_id = created["id"].as_str().unwrap();

    let resp = client
        .put(format!("{}/api/v1/clusters/{cluster_id}", app.address))
        .bearer_auth(&token_reader)
        .json(&json!({
            "name": "Anders",
            "address": "Am Testfeld 1",
            "description": "",
            "soil_condition": "Su3",
            "tree_ids": [],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "visible but not updatable");
}
