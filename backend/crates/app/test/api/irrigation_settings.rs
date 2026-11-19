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

    assert_eq!(body["water_demand"]["value"], 80.0);
    assert_eq!(body["water_demand"]["inherited_value"], 80.0);
    assert_eq!(body["just_watered_ttl_secs"]["value"], 86_400);
}

#[tokio::test]
async fn a_value_set_at_the_root_is_the_inherited_value_below() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    let resp = app
        .put_json(&url(root), &json!({ "water_demand": 120.0 }))
        .await;
    assert_eq!(resp.status(), 200);

    let at_root = irrigation_of(&app, root).await;
    assert_eq!(at_root["water_demand"]["value"], 120.0);
    assert_eq!(at_root["water_demand"]["inherited_value"], 80.0);
    let below = irrigation_of(&app, leaf).await;
    assert_eq!(below["water_demand"]["value"], 120.0);
    assert_eq!(below["water_demand"]["inherited_value"], 120.0);
    assert!(!below.as_object().unwrap().contains_key("enforced_by"));
}

#[tokio::test]
async fn sending_the_inherited_value_falls_back_and_is_recorded() {
    let app = spawn_app().await;
    let org = child_org(&app, "Zurück", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    app.put_json(&url(org), &json!({ "water_demand": 120.0 }))
        .await;
    let resp = app
        .put_json(&url(org), &json!({ "water_demand": 80.0 }))
        .await;
    assert_eq!(resp.status(), 200);

    let stored = sqlx::query_scalar!(
        "SELECT water_demand_liters FROM irrigation_settings WHERE organization_id = $1",
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored, None, "no own value is kept");

    let last = sqlx::query!(
        r#"SELECT previous_value, new_value FROM organization_settings_history
           WHERE organization_id = $1 AND setting_key = 'water_demand'
           ORDER BY changed_at DESC, id DESC LIMIT 1"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(last.previous_value, Some(json!(120.0)));
    assert_eq!(last.new_value, None);
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
