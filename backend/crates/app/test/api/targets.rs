use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app_with_modules},
    module_host::switch,
    test_module::TestModule,
};

const ROOT: &str = "01980000-0000-7000-8000-000000000001";

async fn app() -> TestApp {
    let app = spawn_app_with_modules(vec![Box::new(TestModule::new(Default::default()))]).await;
    switch(&app, ROOT.parse().unwrap(), Some(true)).await;
    app
}

async fn thing(app: &TestApp, label: &str) -> Uuid {
    let res = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": ROOT, "label": label, "latitude": 54.78, "longitude": 9.43 }),
        )
        .await;
    assert_eq!(res.status(), 201);
    res.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

fn thing_target(id: Uuid) -> serde_json::Value {
    json!({ "target": { "module": "test_module", "kind": "thing", "id": id } })
}

#[tokio::test]
async fn a_sensor_attaches_to_another_modules_object() {
    let app = app().await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-thing-1").await;
    let thing = thing(&app, "Behälter 7").await;

    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &thing_target(thing),
        )
        .await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["target"]["module"], "test_module");
    assert_eq!(body["target_label"], "Behälter 7");
    assert!(body["location"].is_object());
}

#[tokio::test]
async fn one_object_carries_at_most_one_sensor() {
    let app = app().await;
    let thing = thing(&app, "a").await;
    let first = crate::sensors::create_prepared_sensor(&app, "eui-a").await;
    let second = crate::sensors::create_prepared_sensor(&app, "eui-b").await;
    let target = thing_target(thing);
    assert_eq!(
        app.post_json(&format!("/api/v1/sensors/{first}/activate"), &target)
            .await
            .status(),
        200
    );

    let res = app
        .post_json(&format!("/api/v1/sensors/{second}/activate"), &target)
        .await;
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["code"],
        "conflict.target_already_has_sensor"
    );
}

#[tokio::test]
async fn a_sensor_on_a_thing_cannot_also_go_onto_a_tree() {
    let app = app().await;
    let thing = thing(&app, "a").await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-c").await;
    app.post_json(
        &format!("/api/v1/sensors/{sensor}/activate"),
        &thing_target(thing),
    )
    .await;

    let res = crate::trees::create_tree_with_sensor(&app, &sensor).await;
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["code"],
        "sensor.already_attached"
    );
}

#[tokio::test]
async fn a_disabled_module_refuses_new_attachments() {
    let app = app().await;
    let thing = thing(&app, "a").await;
    switch(&app, ROOT.parse().unwrap(), Some(false)).await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-d").await;
    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &thing_target(thing),
        )
        .await;
    assert_eq!(res.status(), 403);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["code"],
        "module.disabled"
    );
}

#[tokio::test]
async fn a_kind_that_takes_no_sensors_is_not_accepted() {
    let app = app().await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-e").await;
    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &json!({ "target": { "module": "irrigation", "kind": "cluster", "id": Uuid::now_v7() } }),
        )
        .await;
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["code"],
        "target.not_accepted"
    );
}

#[tokio::test]
async fn deleting_a_tree_frees_its_sensor() {
    let app = crate::helpers::spawn_app().await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-f").await;
    let res = crate::trees::create_tree_with_sensor(&app, &sensor).await;
    assert_eq!(res.status(), 201);
    let tree: serde_json::Value = res.json().await.unwrap();
    app.delete(&format!("/api/v1/trees/{}", tree["id"].as_str().unwrap()))
        .await;

    let body: serde_json::Value = app
        .get(&format!("/api/v1/sensors/{sensor}"))
        .await
        .json()
        .await
        .unwrap();
    assert!(body["target"].is_null());
}

#[tokio::test]
async fn releasing_a_tree_sensor_forgets_the_measured_status() {
    let app = crate::helpers::spawn_app().await;
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-g").await;
    let res = crate::trees::create_tree_with_sensor(&app, &sensor).await;
    let tree: serde_json::Value = res.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();
    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &crate::sensors::tree_target(tree_id),
        )
        .await;
    assert_eq!(res.status(), 200);
    app.set_watering_status(tree_id, "good").await;

    let res = app
        .delete(&format!("/api/v1/sensors/{sensor}/target"))
        .await;
    assert_eq!(res.status(), 200);

    let tree: serde_json::Value = app
        .get(&format!("/api/v1/trees/{tree_id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(tree["watering_status"], "unknown");
}

#[tokio::test]
async fn a_comment_lands_on_another_modules_object() {
    let app = app().await;
    let thing = thing(&app, "Behälter 7").await;
    let url = format!("/api/v1/targets/test_module/thing/{thing}/comments");
    let res = app
        .post_json(&url, &json!({ "body": "Deckel klemmt" }))
        .await;
    assert_eq!(res.status(), 201);
    let list: serde_json::Value = app.get(&url).await.json().await.unwrap();
    assert_eq!(list["data"][0]["body"], "Deckel klemmt");
}

#[tokio::test]
async fn a_tree_is_not_commentable() {
    let app = app().await;
    let res = app
        .post_json(
            &format!(
                "/api/v1/targets/irrigation/tree/{}/comments",
                Uuid::now_v7()
            ),
            &json!({ "body": "x" }),
        )
        .await;
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["code"],
        "target.not_accepted"
    );
}

#[tokio::test]
async fn cluster_comments_are_target_comments() {
    let app = app().await;
    let cluster = crate::clusters::create_cluster(&app).await;
    let res = app
        .post_json(
            &format!("/api/v1/clusters/{cluster}/comments"),
            &json!({ "body": "a" }),
        )
        .await;
    assert_eq!(res.status(), 201);
    let via_target: serde_json::Value = app
        .get(&format!(
            "/api/v1/targets/irrigation/cluster/{cluster}/comments"
        ))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(via_target["data"][0]["body"], "a");
}
