use std::sync::{Arc, atomic::Ordering};
use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::{spawn_app_with_modules, try_spawn_app_with_modules},
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
    test_module::{CollidingModule, KEY, Probe, TestModule},
};

async fn app_with_test_module() -> (crate::helpers::TestApp, Arc<Probe>) {
    let probe = Arc::new(Probe::default());
    let app = spawn_app_with_modules(vec![Box::new(TestModule::new(probe.clone()))]).await;
    (app, probe)
}

pub async fn switch(
    app: &crate::helpers::TestApp,
    org: Uuid,
    enabled: Option<bool>,
) -> reqwest::Response {
    app.put_json(
        &format!("/api/v1/organizations/{org}/modules"),
        &json!({ "modules": { KEY: enabled } }),
    )
    .await
}

#[tokio::test]
async fn a_registered_module_is_listed_and_off_by_default() {
    let (app, _) = app_with_test_module().await;
    let body: serde_json::Value = app.get("/api/v1/modules").await.json().await.unwrap();
    assert_eq!(body["modules"], json!(["irrigation", KEY]));

    let org: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{ROOT_ORG_ID}/modules"))
        .await
        .json()
        .await
        .unwrap();
    let entry = org["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == KEY)
        .unwrap();
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["origin"], "default");
}

#[tokio::test]
async fn writes_into_an_organization_without_the_module_are_refused() {
    let (app, _) = app_with_test_module().await;
    let org = child_org(&app, "Ohne", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let resp = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.disabled");

    assert_eq!(switch(&app, org, Some(true)).await.status(), 200);
    let resp = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn activation_at_the_root_reaches_three_levels_down() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Mitte", root).await;
    let leaf = child_org(&app, "Blatt", mid).await;

    assert_eq!(switch(&app, root, Some(true)).await.status(), 200);

    let body: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{leaf}/modules"))
        .await
        .json()
        .await
        .unwrap();
    let entry = body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == KEY)
        .unwrap();
    assert_eq!(entry["enabled"], true);
    assert_eq!(entry["origin"], "inherited");
    assert_eq!(entry["source"]["id"], ROOT_ORG_ID);
}

#[tokio::test]
async fn a_locked_subtree_cannot_switch_modules() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Sperre", root).await;
    let leaf = child_org(&app, "Darunter", mid).await;
    let lock = app
        .put_json(
            &format!("/api/v1/organizations/{mid}/settings"),
            &json!({ "descendants_may_override": false }),
        )
        .await;
    assert_eq!(lock.status(), 200);

    let resp = switch(&app, leaf, Some(true)).await;
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn reads_only_show_organizations_with_the_module_active() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let on = child_org(&app, "An", root).await;
    let off = child_org(&app, "Aus", root).await;
    assert_eq!(switch(&app, on, Some(true)).await.status(), 200);
    assert_eq!(switch(&app, off, Some(true)).await.status(), 200);
    for org in [on, off] {
        let created = app
            .post_json(
                "/api/v1/test-module/things",
                &json!({ "organization_id": org }),
            )
            .await;
        assert_eq!(created.status(), 201);
    }
    assert_eq!(switch(&app, off, Some(false)).await.status(), 200);

    let things: Vec<serde_json::Value> = app
        .get("/api/v1/test-module/things")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(things.len(), 1);
    assert_eq!(things[0]["organization_id"], on.to_string());
}

#[tokio::test]
async fn module_handlers_receive_core_events_and_tasks_run() {
    let (app, probe) = app_with_test_module().await;
    child_org(&app, "Neu", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;
    assert_eq!(probe.organizations_created.load(Ordering::SeqCst), 1);

    for _ in 0..50 {
        if probe.task_ran.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("background task did not run");
}

#[tokio::test]
async fn a_module_shadowing_a_core_route_stops_the_start() {
    let err = match try_spawn_app_with_modules(vec![Box::new(CollidingModule)]).await {
        Ok(_) => panic!("startup must refuse a colliding module"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("/organizations"), "{err}");
}
