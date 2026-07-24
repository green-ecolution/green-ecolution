use serde_json::json;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method, matchers::path};

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::{TestApp, spawn_app, spawn_app_with_routing};

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

/// Inserts a second child of TBZ, sibling to `SUB_ORG` — a valid share
/// target for TBZ-owned resources that stops being a descendant once the
/// resource moves to `SUB_ORG`.
async fn insert_tbz_sibling_org(app: &TestApp) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(TBZ_ORG).unwrap(),
        "TBZ Schwester-Org",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

async fn mock_streamlet() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    server
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

#[tokio::test]
async fn transfer_cluster_cascades_to_trees_and_sensors() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let sensor_id = "eui-transfer-cluster-a";
    create_sensor(&app, sensor_id, TBZ_ORG).await;
    let tree_id = create_tree(&app, "TRANSFER-CLUSTER-A-001", TBZ_ORG).await;
    activate_sensor(&app, sensor_id, &tree_id).await;
    let cluster_id = create_cluster(&app, &[&tree_id], TBZ_ORG).await;

    let resp = app
        .patch_json(
            &format!("/api/v1/clusters/{cluster_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let cluster_resp = app.get(&format!("/api/v1/clusters/{cluster_id}")).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    assert_eq!(cluster["organization_id"], SUB_ORG);

    let tree_resp = app.get(&format!("/api/v1/trees/{tree_id}")).await;
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    assert_eq!(tree["organization_id"], SUB_ORG);

    let sensor_resp = app.get(&format!("/api/v1/sensors/{sensor_id}")).await;
    let sensor: serde_json::Value = sensor_resp.json().await.unwrap();
    assert_eq!(sensor["organization_id"], SUB_ORG);
}

#[tokio::test]
async fn transfer_cluster_revokes_stale_shares_on_cluster_and_trees() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;
    let tbz_sibling_org = insert_tbz_sibling_org(&app).await;

    // Shared while still clusterless (sharing a clustered tree is rejected),
    // then pulled into a cluster — the share row survives that move because
    // cluster creation updates `trees.tree_cluster_id` directly.
    let tree_id = create_tree(&app, "TRANSFER-CLUSTER-B-001", TBZ_ORG).await;
    let tree_share_resp = app
        .post_json(
            &format!("/api/v1/trees/{tree_id}/shares"),
            &json!({ "organization_id": tbz_sibling_org }),
        )
        .await;
    assert_eq!(tree_share_resp.status(), 204);

    let cluster_id = create_cluster(&app, &[&tree_id], TBZ_ORG).await;
    let cluster_share_resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": tbz_sibling_org }),
        )
        .await;
    assert_eq!(cluster_share_resp.status(), 204);

    let resp = app
        .patch_json(
            &format!("/api/v1/clusters/{cluster_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let cluster_shares: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT organization_id FROM tree_cluster_shares WHERE tree_cluster_id = $1",
        Uuid::parse_str(&cluster_id).unwrap()
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();
    assert!(
        cluster_shares.is_empty(),
        "cluster share pointing outside the new owner's subtree must be revoked"
    );

    let tree_shares: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT organization_id FROM tree_shares WHERE tree_id = $1",
        Uuid::parse_str(&tree_id).unwrap()
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();
    assert!(
        tree_shares.is_empty(),
        "tree share pointing outside the new owner's subtree must be revoked"
    );
}

fn vehicle_payload(plate: &str, org: &str) -> serde_json::Value {
    json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2,
        "width": 2.5,
        "length": 8.0,
        "weight": 12000.0,
        "organization_id": org,
    })
}

#[tokio::test]
async fn transfer_vehicle_returns_204_and_changes_organization() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let resp = app
        .post_json(
            "/api/v1/vehicles",
            &vehicle_payload("FL-TRANSFER 1", TBZ_ORG),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let vehicle: serde_json::Value = resp.json().await.unwrap();
    let vehicle_id = vehicle["id"].as_str().unwrap();

    let resp = app
        .patch_json(
            &format!("/api/v1/vehicles/{vehicle_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let vehicle_resp = app.get(&format!("/api/v1/vehicles/{vehicle_id}")).await;
    let vehicle: serde_json::Value = vehicle_resp.json().await.unwrap();
    assert_eq!(vehicle["organization_id"], SUB_ORG);
}

fn start_point_payload(name: &str, lat: f64, lon: f64, org: &str) -> serde_json::Value {
    json!({ "name": name, "lat": lat, "lon": lon, "watering_point": false, "organization_id": org })
}

#[tokio::test]
async fn transfer_default_start_point_clears_default_and_leaves_target_default_untouched() {
    let streamlet = mock_streamlet().await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    insert_org_tree(&app).await;

    let source: serde_json::Value = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot TBZ", 54.7, 9.4, TBZ_ORG),
        )
        .await
        .json()
        .await
        .unwrap();
    let source_id = source["id"].as_str().unwrap();
    let resp = app
        .post_json(
            &format!("/api/v1/routing/start-points/{source_id}/default"),
            &json!({}),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let target_default: serde_json::Value = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot Unter-Org", 54.75, 9.42, SUB_ORG),
        )
        .await
        .json()
        .await
        .unwrap();
    let target_default_id = target_default["id"].as_str().unwrap();
    let resp = app
        .post_json(
            &format!("/api/v1/routing/start-points/{target_default_id}/default"),
            &json!({}),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let resp = app
        .patch_json(
            &format!("/api/v1/routing/start-points/{source_id}/organization"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let (source_org, source_default): (Uuid, bool) =
        sqlx::query_as("SELECT organization_id, is_default FROM depots WHERE id = $1")
            .bind(Uuid::parse_str(source_id).unwrap())
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(source_org, Uuid::parse_str(SUB_ORG).unwrap());
    assert!(
        !source_default,
        "transferred depot must lose its default status"
    );

    let target_default_still: bool =
        sqlx::query_scalar("SELECT is_default FROM depots WHERE id = $1")
            .bind(Uuid::parse_str(target_default_id).unwrap())
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert!(
        target_default_still,
        "the target org's own default must remain untouched"
    );
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

#[tokio::test]
async fn transfer_tree_invisible_to_foreign_org_returns_404() {
    let (harness, app) = spawn_with_auth().await;
    let sibling_org = insert_org_tree(&app).await;

    let admin_token = seed_admin(&harness, &app, Uuid::parse_str(TBZ_ORG).unwrap()).await;
    let tree_resp = reqwest::Client::new()
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&admin_token)
        .json(&tree_payload("TRANSFER-F-001", TBZ_ORG))
        .send()
        .await
        .unwrap();
    assert_eq!(tree_resp.status(), 201);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();

    // The sibling org's admin has tree:update, just not over TBZ_ORG's
    // subtree — the tree must be indistinguishable from a missing one.
    let foreign_admin_token = seed_admin(&harness, &app, sibling_org).await;
    let resp = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/trees/{tree_id}/organization",
            app.address
        ))
        .bearer_auth(&foreign_admin_token)
        .json(&json!({ "organization_id": sibling_org }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "invisible tree must read as 404");
}
