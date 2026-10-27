use std::sync::Arc;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth_helpers::AuthHarness,
    helpers::{TestApp, seed_user_with_permissions, spawn_app_with_modules_and_auth},
    test_module::{KEY, Probe, TestModule},
};

async fn app_with_auth() -> (AuthHarness, TestApp) {
    let harness = AuthHarness::start().await;
    let module = TestModule::new(Arc::new(Probe::default()));
    let app =
        spawn_app_with_modules_and_auth(vec![Box::new(module)], harness.auth_settings(true)).await;
    (harness, app)
}

async fn child_of(app: &TestApp, parent: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1, $2) RETURNING id",
    )
    .bind(parent)
    .bind(name)
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

async fn activate(app: &TestApp, org: Uuid) {
    sqlx::query(
        "INSERT INTO organization_modules (organization_id, module_key, enabled) VALUES ($1, $2, true)",
    )
    .bind(org)
    .bind(KEY)
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_thing(app: &TestApp, org: Uuid) {
    sqlx::query("INSERT INTO test_module_things (id, organization_id) VALUES ($1, $2)")
        .bind(Uuid::now_v7())
        .bind(org)
        .execute(&app.db_pool)
        .await
        .unwrap();
}

fn switch_body() -> Value {
    json!({ "modules": { KEY: true } })
}

#[tokio::test]
async fn the_module_list_only_covers_the_callers_subtree() {
    let (harness, app) = app_with_auth().await;
    let (a, token) = seed_user_with_permissions(&harness, &app, "A", &["setting:read"]).await;
    let (b, _) = seed_user_with_permissions(&harness, &app, "B", &["setting:read"]).await;
    let child = child_of(&app, a, "A-Kind").await;

    let resp = app.get_with_bearer("/api/v1/modules", &token).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["modules"], json!(["irrigation", KEY, "waste"]));
    let orgs: Vec<String> = body["organizations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["organization_id"].as_str().unwrap().to_string())
        .collect();
    assert!(orgs.contains(&a.to_string()), "{orgs:?}");
    assert!(orgs.contains(&child.to_string()), "{orgs:?}");
    assert!(!orgs.contains(&b.to_string()), "{orgs:?}");
}

#[tokio::test]
async fn switching_modules_needs_setting_update() {
    let (harness, app) = app_with_auth().await;
    let (a, token) = seed_user_with_permissions(&harness, &app, "A", &["organization:read"]).await;

    let resp = app
        .put_json_with_bearer(
            &format!("/api/v1/organizations/{a}/modules"),
            &switch_body(),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn a_reader_may_look_but_not_switch() {
    let (harness, app) = app_with_auth().await;
    let (a, token) = seed_user_with_permissions(&harness, &app, "A", &["setting:read"]).await;

    let read = app
        .get_with_bearer(&format!("/api/v1/organizations/{a}/modules"), &token)
        .await;
    assert_eq!(read.status(), 200);
    let write = app
        .put_json_with_bearer(
            &format!("/api/v1/organizations/{a}/modules"),
            &switch_body(),
            &token,
        )
        .await;
    assert_eq!(write.status(), 403);
}

#[tokio::test]
async fn a_module_route_without_a_token_is_unauthorized() {
    let (_harness, app) = app_with_auth().await;
    let resp = reqwest::get(format!("{}/api/v1/test-module/things", app.address))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn module_reads_intersect_grants_with_activation() {
    let (harness, app) = app_with_auth().await;
    let (a, token) = seed_user_with_permissions(&harness, &app, "A", &["organization:read"]).await;
    let (b, _) = seed_user_with_permissions(&harness, &app, "B", &["organization:read"]).await;
    for org in [a, b] {
        activate(&app, org).await;
        insert_thing(&app, org).await;
    }

    let resp = app
        .get_with_bearer("/api/v1/test-module/things", &token)
        .await;
    assert_eq!(resp.status(), 200);
    let things: Vec<Value> = resp.json().await.unwrap();
    assert_eq!(things.len(), 1);
    assert_eq!(things[0]["organization_id"], a.to_string());
}
