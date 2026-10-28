use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::TestApp, organizations::ROOT_ORG_ID, settings_repo::child_org, waste_bins::waste_app,
};

fn url(org: impl std::fmt::Display) -> String {
    format!("/api/v1/organizations/{org}/settings/waste")
}

fn root() -> Uuid {
    Uuid::parse_str(ROOT_ORG_ID).unwrap()
}

async fn waste_of(app: &TestApp, org: Uuid) -> serde_json::Value {
    let resp = app.get(&url(org)).await;
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

#[tokio::test]
async fn the_default_comes_from_the_instance_config() {
    let app = waste_app().await;

    let body = waste_of(&app, root()).await;

    assert_eq!(body["full_threshold"]["origin"], "default");
    assert_eq!(body["full_threshold"]["value"], 80);
    assert!(body["enforced_by"].is_null());
}

#[tokio::test]
async fn a_set_value_is_own_at_the_root_and_inherited_below() {
    let app = waste_app().await;
    let child = child_org(&app, "Kind", root()).await;

    let res = app
        .put_json(&url(root()), &json!({ "full_threshold": 70 }))
        .await;
    assert_eq!(res.status(), 200);

    assert_eq!(
        waste_of(&app, root()).await["full_threshold"]["origin"],
        "own"
    );
    let below = waste_of(&app, child).await;
    assert_eq!(below["full_threshold"]["value"], 70);
    assert_eq!(below["full_threshold"]["origin"], "inherited");
    assert_eq!(below["full_threshold"]["source"]["id"], root().to_string());
}

#[tokio::test]
async fn a_locked_ancestor_refuses_waste_writes_below() {
    let app = waste_app().await;
    let child = child_org(&app, "Kind", root()).await;
    app.put_json(
        &format!("/api/v1/organizations/{}/settings", root()),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let res = app
        .put_json(&url(child), &json!({ "full_threshold": 60 }))
        .await;

    assert_eq!(res.status(), 409);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"], "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn clearing_a_value_falls_back_and_is_recorded() {
    let app = waste_app().await;
    app.put_json(&url(root()), &json!({ "full_threshold": 60 }))
        .await;
    app.put_json(&url(root()), &json!({ "full_threshold": null }))
        .await;

    let body = waste_of(&app, root()).await;

    assert_eq!(body["full_threshold"]["origin"], "default");
    assert!(body["full_threshold"]["last_change"].is_object());
}

#[tokio::test]
async fn an_out_of_range_value_answers_400_with_a_field_label() {
    let app = waste_app().await;

    for bad in [0, 101] {
        let res = app
            .put_json(&url(root()), &json!({ "full_threshold": bad }))
            .await;

        assert_eq!(res.status(), 400);
        let body: serde_json::Value = res.json().await.unwrap();
        assert_eq!(body["validation"]["field"], "settings.full_threshold");
    }
}

#[tokio::test]
async fn an_unknown_organization_answers_404() {
    let app = waste_app().await;
    let unknown = Uuid::now_v7();

    assert_eq!(app.get(&url(unknown)).await.status(), 404);
    assert_eq!(
        app.put_json(&url(unknown), &json!({ "full_threshold": 70 }))
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn changes_are_written_to_the_shared_settings_history() {
    let app = waste_app().await;
    app.put_json(&url(root()), &json!({ "full_threshold": 70 }))
        .await;
    app.put_json(&url(root()), &json!({ "full_threshold": 90 }))
        .await;

    let rows = sqlx::query!(
        r#"SELECT setting_key AS "key!", previous_value, new_value
           FROM organization_settings_history
           WHERE organization_id = $1 AND setting_key = 'full_threshold'
           ORDER BY changed_at ASC, id ASC"#,
        root()
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].previous_value, Some(json!(70)));
    assert_eq!(rows[1].new_value, Some(json!(90)));
}

#[tokio::test]
async fn the_inherited_threshold_drives_the_fill_state_of_a_bin() {
    let app = waste_app().await;
    app.put_json(&url(root()), &json!({ "full_threshold": 50 }))
        .await;
    let bin = crate::waste_bins::create_bin(&app, "B-1").await;
    let id: Uuid = bin["id"].as_str().unwrap().parse().unwrap();
    sqlx::query(
        "UPDATE waste_bins SET fill_fraction = 0.6, fill_measured_at = now() WHERE id = $1",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .unwrap();

    let body: serde_json::Value = app
        .get(&format!("/api/v1/waste-bins/{id}"))
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["fill_state"], "full");
}
