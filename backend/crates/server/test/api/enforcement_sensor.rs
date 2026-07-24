use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::{TestApp, seed_user_with_permissions};

fn sensor_payload(id: &str, model_id: Uuid, org: Uuid) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "organization_id": org,
        "lorawan": {
            "serial_number": "SN-ENFORCE",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

fn tree_payload(number: &str, org: Uuid) -> serde_json::Value {
    json!({
        "species": "Tilia", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": "",
        "organization_id": org,
    })
}

/// Seeds an org below `parent` (not necessarily ROOT), a role with exactly
/// `permissions`, and a user holding it. Needed on top of
/// `seed_user_with_permissions` (which always parents under ROOT) so a share
/// target can be a genuine descendant of the owning org.
async fn seed_child_org_with_permissions(
    harness: &AuthHarness,
    app: &TestApp,
    parent: Uuid,
    org_name: &str,
    permissions: &[&str],
) -> (Uuid, String) {
    let org_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1, $2) RETURNING id"#,
        parent,
        org_name,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Test-Rolle', $2)
           RETURNING id"#,
        org_id,
        &permissions,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org_id
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
    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    (org_id, token)
}

#[tokio::test]
async fn foreign_org_sees_no_sensors_and_gets_404_on_detail() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", &["sensor:read", "sensor:create"])
            .await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["sensor:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-001", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b["data"].as_array().unwrap().len(), 0);

    let detail_b = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_a["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn sensor_on_shared_tree_is_visible_to_target_org() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) = seed_user_with_permissions(
        &harness,
        &app,
        "Owner Org",
        &[
            "sensor:read",
            "sensor:create",
            "sensor:update",
            "tree:read",
            "tree:create",
            "tree:update",
        ],
    )
    .await;
    let (org_b, token_b) = seed_child_org_with_permissions(
        &harness,
        &app,
        org_a,
        "Target Org",
        &["sensor:read", "tree:read"],
    )
    .await;
    let client = reqwest::Client::new();

    let sensor: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-002", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap().to_owned();

    let tree: serde_json::Value = client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token_a)
        .json(&tree_payload("ENFORCE-002", org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tree_id = tree["id"].as_str().unwrap().to_owned();

    let activate_resp = client
        .post(format!(
            "{}/api/v1/sensors/{sensor_id}/activate",
            app.address
        ))
        .bearer_auth(&token_a)
        .json(&json!({ "tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(activate_resp.status(), 200);

    let share_resp = client
        .post(format!("{}/api/v1/trees/{tree_id}/shares", app.address))
        .bearer_auth(&token_a)
        .json(&json!({ "organization_id": org_b }))
        .send()
        .await
        .unwrap();
    assert_eq!(share_resp.status(), 204);

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        list_b["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == sensor_id),
        "sensor linked to a shared tree must be visible to the target org"
    );

    let detail_b = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 200);
}

#[tokio::test]
async fn activate_by_user_without_tree_update_yields_403() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Org",
        &[
            "sensor:read",
            "sensor:create",
            "sensor:update",
            "tree:read",
            "tree:create",
        ],
    )
    .await;
    let client = reqwest::Client::new();

    let sensor: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token)
        .json(&sensor_payload("eui-enforce-003", model_id, org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap().to_owned();

    let tree: serde_json::Value = client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token)
        .json(&tree_payload("ENFORCE-003", org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tree_id = tree["id"].as_str().unwrap().to_owned();

    let activate_resp = client
        .post(format!(
            "{}/api/v1/sensors/{sensor_id}/activate",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        activate_resp.status(),
        403,
        "sensor:update alone must not be enough - tree:update is also required"
    );
}

#[tokio::test]
async fn delete_by_target_org_is_forbidden_despite_share_and_delete_permission() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) = seed_user_with_permissions(
        &harness,
        &app,
        "Owner Org",
        &[
            "sensor:read",
            "sensor:create",
            "sensor:update",
            "tree:read",
            "tree:create",
            "tree:update",
        ],
    )
    .await;
    let (org_b, token_b) = seed_child_org_with_permissions(
        &harness,
        &app,
        org_a,
        "Target Org",
        &["sensor:read", "sensor:delete"],
    )
    .await;
    let client = reqwest::Client::new();

    let sensor: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-004", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap().to_owned();

    let tree: serde_json::Value = client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token_a)
        .json(&tree_payload("ENFORCE-004", org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tree_id = tree["id"].as_str().unwrap().to_owned();

    let activate_resp = client
        .post(format!(
            "{}/api/v1/sensors/{sensor_id}/activate",
            app.address
        ))
        .bearer_auth(&token_a)
        .json(&json!({ "tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(activate_resp.status(), 200);

    let share_resp = client
        .post(format!("{}/api/v1/trees/{tree_id}/shares", app.address))
        .bearer_auth(&token_a)
        .json(&json!({ "organization_id": org_b }))
        .send()
        .await
        .unwrap();
    assert_eq!(share_resp.status(), 204);

    let delete_resp = client
        .delete(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        delete_resp.status(),
        403,
        "delete must stay owner-org-only even for a shared, delete-permitted target org"
    );
}

/// Covers the inheritance chain cluster-share -> tree-in-cluster ->
/// coupled sensor (the `t.tree_cluster_id IN (... tree_cluster_shares ...)`
/// branch in `pg_sensor.rs`), which only `sensor_on_shared_tree_is_visible_to_target_org`
/// (direct tree share) exercised so far.
#[tokio::test]
async fn sensor_on_shared_cluster_is_visible_to_target_org() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) = seed_user_with_permissions(
        &harness,
        &app,
        "Owner Org",
        &[
            "sensor:read",
            "sensor:create",
            "sensor:update",
            "tree:read",
            "tree:create",
            "tree:update",
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let (org_b, token_b) =
        seed_child_org_with_permissions(&harness, &app, org_a, "Target Org", &["sensor:read"])
            .await;
    let client = reqwest::Client::new();

    let tree: serde_json::Value = client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token_a)
        .json(&tree_payload("ENFORCE-006", org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tree_id = tree["id"].as_str().unwrap().to_owned();

    let cluster: serde_json::Value = client
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&token_a)
        .json(&json!({
            "name": "Enforcement Cluster",
            "address": "Am Testfeld 1",
            "description": "",
            "soil_condition": "Su3",
            "tree_ids": [tree_id],
            "organization_id": org_a,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cluster_id = cluster["id"].as_str().unwrap().to_owned();

    let sensor: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-006", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap().to_owned();

    let activate_resp = client
        .post(format!(
            "{}/api/v1/sensors/{sensor_id}/activate",
            app.address
        ))
        .bearer_auth(&token_a)
        .json(&json!({ "tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(activate_resp.status(), 200);

    app.ingest_ecodrizzler("eui-enforce-006", 45).await.unwrap();

    // Negative baseline: before the cluster share exists, the target org
    // must not see the sensor at all.
    let list_before: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        list_before["data"].as_array().unwrap().len(),
        0,
        "sensor coupled to a tree in an unshared cluster must not be visible yet"
    );

    let detail_before = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_before.status(), 404);

    let share_resp = client
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/shares",
            app.address
        ))
        .bearer_auth(&token_a)
        .json(&json!({ "organization_id": org_b }))
        .send()
        .await
        .unwrap();
    assert_eq!(share_resp.status(), 204);

    let list_after: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        list_after["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == sensor_id),
        "sensor linked to a tree in a shared cluster must be visible to the target org"
    );

    let detail_after: serde_json::Value = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        detail_after["shared_with"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == &json!(org_b)),
        "shared_with must include the cluster-share target org"
    );

    let data_resp = client
        .get(format!("{}/api/v1/sensors/{sensor_id}/data", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        data_resp.status(),
        200,
        "sensor data must be reachable through the inherited cluster share"
    );

    let revoke_resp = client
        .delete(format!(
            "{}/api/v1/clusters/{cluster_id}/shares/{org_b}",
            app.address
        ))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap();
    assert_eq!(revoke_resp.status(), 204);

    let detail_revoked = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        detail_revoked.status(),
        404,
        "revoking the cluster share must hide the sensor again"
    );
}

#[tokio::test]
async fn sensor_data_endpoint_is_404_for_foreign_org() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", &["sensor:read", "sensor:create"])
            .await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["sensor:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-005", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = created["id"].as_str().unwrap().to_owned();

    app.ingest_ecodrizzler("eui-enforce-005", 45).await.unwrap();

    let data_resp_a = client
        .get(format!("{}/api/v1/sensors/{sensor_id}/data", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap();
    assert_eq!(data_resp_a.status(), 200);

    let data_resp_b = client
        .get(format!("{}/api/v1/sensors/{sensor_id}/data", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(data_resp_b.status(), 404);
}
