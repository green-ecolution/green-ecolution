use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app},
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
};

fn url(org: impl std::fmt::Display) -> String {
    format!("/api/v1/organizations/{org}/settings/irrigation")
}

async fn irrigation_of(app: &TestApp, org: Uuid) -> serde_json::Value {
    let resp = app.get(&url(org)).await;
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

#[tokio::test]
async fn defaults_come_from_the_instance_config() {
    let app = spawn_app().await;

    let body = irrigation_of(&app, Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    assert_eq!(body["water_demand"]["origin"], "default");
    assert_eq!(body["water_demand"]["value"], 80.0);
    assert_eq!(body["just_watered_ttl_secs"]["origin"], "default");
    assert!(body["enforced_by"].is_null());
}

#[tokio::test]
async fn a_set_value_is_own_at_the_root_and_inherited_below() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let child = child_org(&app, "Kind", root).await;

    let res = app
        .put_json(&url(root), &json!({ "water_demand": 42.0 }))
        .await;
    assert_eq!(res.status(), 200);

    let own = irrigation_of(&app, root).await;
    assert_eq!(own["water_demand"]["origin"], "own");

    let below = irrigation_of(&app, child).await;
    assert_eq!(below["water_demand"]["value"], 42.0);
    assert_eq!(below["water_demand"]["origin"], "inherited");
    assert_eq!(below["water_demand"]["source"]["id"], root.to_string());
}

#[tokio::test]
async fn a_locked_ancestor_refuses_irrigation_writes_below() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let child = child_org(&app, "Kind", root).await;
    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let res = app
        .put_json(&url(child), &json!({ "water_demand": 10.0 }))
        .await;

    assert_eq!(res.status(), 409);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"], "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn the_lock_freezes_a_dormant_own_value_and_reports_it() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let child = child_org(&app, "Kind", root).await;
    app.put_json(&url(root), &json!({ "water_demand": 100.0 }))
        .await;
    app.put_json(&url(child), &json!({ "water_demand": 70.0 }))
        .await;
    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let body = irrigation_of(&app, child).await;

    assert_eq!(body["water_demand"]["value"], 100.0);
    assert_eq!(body["water_demand"]["own_value"], 70.0);
    assert_eq!(body["enforced_by"]["id"], root.to_string());
}

#[tokio::test]
async fn clearing_a_value_falls_back_and_is_recorded() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    app.put_json(&url(root), &json!({ "just_watered_ttl_secs": 7200 }))
        .await;
    app.put_json(&url(root), &json!({ "just_watered_ttl_secs": null }))
        .await;

    let body = irrigation_of(&app, root).await;

    assert_eq!(body["just_watered_ttl_secs"]["origin"], "default");
    assert!(body["just_watered_ttl_secs"]["last_change"].is_object());
    assert!(body["water_demand"]["last_change"].is_null());
}

#[tokio::test]
async fn an_omitted_field_is_left_alone() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    app.put_json(
        &url(root),
        &json!({ "water_demand": 70.0, "just_watered_ttl_secs": 7200 }),
    )
    .await;
    app.put_json(&url(root), &json!({ "just_watered_ttl_secs": 10_800 }))
        .await;

    let body = irrigation_of(&app, root).await;

    assert_eq!(body["water_demand"]["value"], 70.0);
    assert_eq!(body["just_watered_ttl_secs"]["value"], 10_800);
}

#[tokio::test]
async fn an_out_of_range_value_answers_400_with_a_field_label() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();

    let res = app
        .put_json(&url(root), &json!({ "water_demand": 100000.0 }))
        .await;

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["validation"]["field"], "settings.water_demand");
}

#[tokio::test]
async fn an_unknown_organization_answers_404() {
    let app = spawn_app().await;
    let unknown = Uuid::now_v7();

    assert_eq!(app.get(&url(unknown)).await.status(), 404);
    assert_eq!(
        app.put_json(&url(unknown), &json!({ "water_demand": 70.0 }))
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn changes_are_written_to_the_shared_settings_history() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    app.put_json(&url(root), &json!({ "water_demand": 70.0 }))
        .await;
    app.put_json(&url(root), &json!({ "water_demand": 90.0 }))
        .await;

    let rows = sqlx::query!(
        r#"SELECT setting_key AS "key!", previous_value, new_value
           FROM organization_settings_history
           WHERE organization_id = $1
           ORDER BY changed_at ASC, id ASC"#,
        root
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].key, "water_demand");
    assert_eq!(rows[1].previous_value, Some(json!(70.0)));
    assert_eq!(rows[1].new_value, Some(json!(90.0)));
}

#[tokio::test]
async fn the_core_settings_no_longer_carry_irrigation_fields() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();

    let body: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{root}/settings"))
        .await
        .json()
        .await
        .unwrap();

    assert!(body.get("water_demand").is_none());
    assert!(body.get("just_watered_ttl_secs").is_none());
}
