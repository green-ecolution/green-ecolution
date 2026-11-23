use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth_helpers::{AuthHarness, spawn_with_auth},
    helpers::{TestApp, seed_user_with_permissions, set_module},
};

const PERMISSIONS: &[&str] = &[
    "tree:create",
    "tree:read",
    "tree:update",
    "tree:delete",
    "tree_cluster:create",
    "tree_cluster:read",
    "tree_cluster:update",
    "tree_cluster:delete",
    "watering_plan:read",
    "sensor:read",
    "vehicle:read",
];

struct Fixture {
    app: TestApp,
    tenant: Uuid,
    token: String,
    tree: String,
    cluster: String,
}

fn tree_payload(number: &str) -> Value {
    json!({
        "species": "Quercus robur", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": ""
    })
}

fn cluster_payload(tree_ids: &[&str]) -> Value {
    json!({
        "name": "Access Cluster", "address": "Am Testfeld 1", "description": "",
        "soil_condition": "Su3", "tree_ids": tree_ids,
    })
}

async fn created_id(resp: reqwest::Response) -> String {
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn fixture_in(harness: &AuthHarness, app: TestApp) -> Fixture {
    let (tenant, token) = seed_user_with_permissions(harness, &app, "Access", PERMISSIONS).await;
    let tree = created_id(
        app.post_json_with_bearer("/api/v1/trees", &tree_payload("ACC-001"), &token)
            .await,
    )
    .await;
    let clustered_tree = created_id(
        app.post_json_with_bearer("/api/v1/trees", &tree_payload("ACC-002"), &token)
            .await,
    )
    .await;
    let cluster = created_id(
        app.post_json_with_bearer(
            "/api/v1/clusters",
            &cluster_payload(&[clustered_tree.as_str()]),
            &token,
        )
        .await,
    )
    .await;
    Fixture {
        app,
        tenant,
        token,
        tree,
        cluster,
    }
}

async fn switched_on() -> (AuthHarness, Fixture) {
    let (harness, app) = spawn_with_auth().await;
    let fixture = fixture_in(&harness, app).await;
    (harness, fixture)
}

async fn switched_off() -> (AuthHarness, Fixture) {
    let (harness, fixture) = switched_on().await;
    set_module(&fixture.app, fixture.tenant, "irrigation", true, false).await;
    (harness, fixture)
}

impl Fixture {
    async fn get(&self, path: &str) -> reqwest::Response {
        self.app.get_with_bearer(path, &self.token).await
    }

    async fn tree_count(&self) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM trees WHERE organization_id = $1")
            .bind(self.tenant)
            .fetch_one(&self.app.db_pool)
            .await
            .unwrap()
    }
}

async fn json_of(resp: reqwest::Response) -> Value {
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

async fn assert_module_disabled(resp: reqwest::Response) {
    assert_eq!(resp.status(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.disabled");
}

#[tokio::test]
async fn lists_are_empty_while_irrigation_is_off() {
    let (_harness, f) = switched_off().await;
    let bbox = "bbox=54.78,9.40,54.81,9.46";

    for path in ["/api/v1/trees", "/api/v1/clusters"] {
        let body = json_of(f.get(path).await).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 0, "{path}");
    }
    for path in [
        format!("/api/v1/trees/markers?{bbox}"),
        "/api/v1/clusters/markers".to_string(),
        "/api/v1/clusters/boundaries".to_string(),
    ] {
        let body = json_of(f.get(&path).await).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 0, "{path}");
    }
    let years = json_of(f.get("/api/v1/trees/planting-years").await).await;
    assert_eq!(years.as_array().unwrap().len(), 0);
    let nearest = json_of(f.get("/api/v1/trees/nearest?lat=54.79&lng=9.44").await).await;
    assert_eq!(nearest["data"].as_array().unwrap().len(), 0);
    let statistics = json_of(f.get("/api/v1/clusters/statistics").await).await;
    assert_eq!(statistics["total"], 0);
}

#[tokio::test]
async fn details_read_as_not_found_while_irrigation_is_off() {
    let (_harness, f) = switched_off().await;
    for path in [
        format!("/api/v1/trees/{}", f.tree),
        format!("/api/v1/clusters/{}", f.cluster),
        format!("/api/v1/clusters/{}/sensor-ids", f.cluster),
        format!("/api/v1/clusters/{}/soil-moisture", f.cluster),
        format!("/api/v1/clusters/{}/comments", f.cluster),
    ] {
        assert_eq!(f.get(&path).await.status(), 404, "{path}");
    }
}

#[tokio::test]
async fn writes_are_refused_while_irrigation_is_off() {
    let (_harness, f) = switched_off().await;
    let app = &f.app;
    let token = f.token.as_str();

    assert_module_disabled(
        app.post_json_with_bearer("/api/v1/trees", &tree_payload("ACC-NEW"), token)
            .await,
    )
    .await;
    assert_module_disabled(
        app.post_json_with_bearer("/api/v1/clusters", &cluster_payload(&[]), token)
            .await,
    )
    .await;
    assert_eq!(f.tree_count().await, 2);

    for resp in [
        app.put_json_with_bearer(
            &format!("/api/v1/trees/{}", f.tree),
            &tree_payload("ACC-001"),
            token,
        )
        .await,
        app.delete_with_bearer(&format!("/api/v1/trees/{}", f.tree), token)
            .await,
        app.put_json_with_bearer(
            &format!("/api/v1/clusters/{}", f.cluster),
            &cluster_payload(&[]),
            token,
        )
        .await,
        app.delete_with_bearer(&format!("/api/v1/clusters/{}", f.cluster), token)
            .await,
    ] {
        assert_module_disabled(resp).await;
    }
    assert_eq!(f.tree_count().await, 2);
}

#[tokio::test]
async fn a_transfer_needs_irrigation_in_source_and_target() {
    let (_harness, f) = switched_on().await;
    let target: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (id, parent_id, name)
         VALUES (gen_random_uuid(), $1, 'Target') RETURNING id",
    )
    .bind(Uuid::parse_str("01980000-0000-7000-8000-000000000001").unwrap())
    .fetch_one(&f.app.db_pool)
    .await
    .unwrap();
    let role: Uuid = sqlx::query_scalar(
        "INSERT INTO roles (id, organization_id, name, permissions)
         VALUES (gen_random_uuid(), $1, 'Target-Rolle', $2) RETURNING id",
    )
    .bind(target)
    .bind(vec!["tree:update".to_string()])
    .fetch_one(&f.app.db_pool)
    .await
    .unwrap();
    let caller: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM role_assignments ra JOIN roles r ON r.id = ra.role_id
         WHERE r.organization_id = $1 LIMIT 1",
    )
    .bind(f.tenant)
    .fetch_one(&f.app.db_pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)")
        .bind(caller)
        .bind(role)
        .execute(&f.app.db_pool)
        .await
        .unwrap();
    set_module(&f.app, target, "irrigation", true, false).await;

    let resp = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/trees/{}/organization",
            f.app.address, f.tree
        ))
        .bearer_auth(&f.token)
        .json(&json!({ "organization_id": target }))
        .send()
        .await
        .unwrap();
    assert_module_disabled(resp).await;
}

#[tokio::test]
async fn the_evaluation_counts_nothing_while_irrigation_is_off() {
    let (_harness, f) = switched_on().await;
    let before = json_of(f.get("/api/v1/evaluation").await).await;
    assert_eq!(before["tree_count"], 2);
    assert_eq!(before["treecluster_count"], 1);

    set_module(&f.app, f.tenant, "irrigation", true, false).await;
    let after = json_of(f.get("/api/v1/evaluation").await).await;
    assert_eq!(after["tree_count"], 0);
    assert_eq!(after["treecluster_count"], 0);
}

#[tokio::test]
async fn everything_answers_403_while_irrigation_is_locked() {
    let (_harness, f) = switched_on().await;
    set_module(&f.app, f.tenant, "irrigation", false, true).await;
    assert_module_disabled(f.get("/api/v1/trees").await).await;
    assert_module_disabled(f.get(&format!("/api/v1/trees/{}", f.tree)).await).await;
}

#[tokio::test]
async fn irrigation_works_as_before_while_switched_on() {
    let (_harness, f) = switched_on().await;
    let list = json_of(f.get("/api/v1/trees").await).await;
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
    assert_eq!(
        f.get(&format!("/api/v1/trees/{}", f.tree)).await.status(),
        200
    );
    assert_eq!(
        f.get(&format!("/api/v1/clusters/{}", f.cluster))
            .await
            .status(),
        200
    );
}
