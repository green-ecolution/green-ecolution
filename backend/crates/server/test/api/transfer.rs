use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::{TestApp, spawn_app};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";
const SUB_ORG: &str = "01980000-0000-7000-8000-000000000003";

/// Returns the sibling org's id (parented directly at root, alongside TBZ).
async fn insert_org_tree(app: &TestApp) -> Uuid {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ')",
        Uuid::parse_str(TBZ_ORG).unwrap(),
        Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ Unter-Org')",
        Uuid::parse_str(SUB_ORG).unwrap(),
        Uuid::parse_str(TBZ_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(ROOT_ORG).unwrap(),
        "Geschwister-Org",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

fn tree_payload(number: &str, org: &str) -> serde_json::Value {
    json!({
        "species": "Quercus robur", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": "",
        "organization_id": org,
    })
}

fn cluster_payload(tree_ids: &[&str], org: &str) -> serde_json::Value {
    json!({
        "name": "Transfer Cluster",
        "address": "Am Testfeld 1",
        "description": "",
        "soil_condition": "Su3",
        "tree_ids": tree_ids,
        "organization_id": org,
    })
}

fn sensor_payload(id: &str, model_id: Uuid, org: &str) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "organization_id": org,
        "lorawan": {
            "serial_number": "SN-TRANSFER",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

async fn create_tree(app: &TestApp, number: &str, org: &str) -> String {
    let resp = app
        .post_json("/api/v1/trees", &tree_payload(number, org))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_owned()
}

async fn create_cluster(app: &TestApp, tree_ids: &[&str], org: &str) -> String {
    let resp = app
        .post_json("/api/v1/clusters", &cluster_payload(tree_ids, org))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_owned()
}

async fn create_sensor(app: &TestApp, id: &str, org: &str) {
    let model_id = app.ecodrizzler_model_id().await;
    let resp = app
        .post_json("/api/v1/sensors", &sensor_payload(id, model_id, org))
        .await;
    assert_eq!(resp.status(), 201);
}

async fn activate_sensor(app: &TestApp, sensor_id: &str, tree_id: &str) {
    let resp = app
        .post_json(
            &format!("/api/v1/sensors/{sensor_id}/activate"),
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn transfer_clusterless_tree_cascades_attached_sensor() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let sensor_id = "eui-transfer-a";
    create_sensor(&app, sensor_id, TBZ_ORG).await;
    let tree_id = create_tree(&app, "TRANSFER-A-001", TBZ_ORG).await;
    activate_sensor(&app, sensor_id, &tree_id).await;

    let resp = app
        .patch_json(
            &format!("/api/v1/trees/{tree_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let tree_resp = app.get(&format!("/api/v1/trees/{tree_id}")).await;
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    assert_eq!(tree["organization_id"], SUB_ORG);

    let sensor_resp = app.get(&format!("/api/v1/sensors/{sensor_id}")).await;
    let sensor: serde_json::Value = sensor_resp.json().await.unwrap();
    assert_eq!(sensor["organization_id"], SUB_ORG);
}

#[tokio::test]
async fn transfer_tree_in_cluster_returns_409() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let tree_id = create_tree(&app, "TRANSFER-B-001", TBZ_ORG).await;
    let _cluster_id = create_cluster(&app, &[&tree_id], TBZ_ORG).await;

    let resp = app
        .patch_json(
            &format!("/api/v1/trees/{tree_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn transfer_bound_sensor_directly_returns_409() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let sensor_id = "eui-transfer-c";
    create_sensor(&app, sensor_id, TBZ_ORG).await;
    let tree_id = create_tree(&app, "TRANSFER-C-001", TBZ_ORG).await;
    activate_sensor(&app, sensor_id, &tree_id).await;

    let resp = app
        .patch_json(
            &format!("/api/v1/sensors/{sensor_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn transfer_unbound_sensor_returns_204() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let sensor_id = "eui-transfer-d";
    create_sensor(&app, sensor_id, TBZ_ORG).await;

    let resp = app
        .patch_json(
            &format!("/api/v1/sensors/{sensor_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let sensor_resp = app.get(&format!("/api/v1/sensors/{sensor_id}")).await;
    let sensor: serde_json::Value = sensor_resp.json().await.unwrap();
    assert_eq!(sensor["organization_id"], SUB_ORG);
}

/// Seeds an admin-copy role (full permission set) in `org_id` and a user
/// holding it, mirroring `sharing.rs::seed_admin`.
async fn seed_admin(
    harness: &crate::auth_helpers::AuthHarness,
    app: &TestApp,
    org_id: Uuid,
) -> String {
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           SELECT gen_random_uuid(), $1, 'Org-Admin', permissions FROM roles WHERE id = '01980000-0000-7000-8000-0000000000a1'
           RETURNING id"#,
        org_id
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
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

#[tokio::test]
async fn transfer_requires_update_permission_in_the_target_organization() {
    let (harness, app) = spawn_with_auth().await;
    let sibling_org = insert_org_tree(&app).await;

    let admin_token = seed_admin(&harness, &app, Uuid::parse_str(TBZ_ORG).unwrap()).await;

    let tree_resp = reqwest::Client::new()
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&admin_token)
        .json(&tree_payload("TRANSFER-E-001", TBZ_ORG))
        .send()
        .await
        .unwrap();
    assert_eq!(tree_resp.status(), 201);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();

    // The admin has tree:update in TBZ_ORG (source) and its subtree, but the
    // sibling org sits outside that subtree, so the target-org check fails.
    let resp = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/trees/{tree_id}/organization",
            app.address
        ))
        .bearer_auth(&admin_token)
        .json(&json!({ "organization_id": sibling_org }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
