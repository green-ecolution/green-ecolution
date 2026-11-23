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

    assert_eq!(body["full_threshold"]["value"], 80);
    assert_eq!(body["full_threshold"]["inherited_value"], 80);
}

#[tokio::test]
async fn a_value_set_at_the_root_is_the_inherited_value_below() {
    let app = waste_app().await;
    let leaf = child_org(&app, "Blatt", root()).await;

    let resp = app
        .put_json(&url(root()), &json!({ "full_threshold": 60 }))
        .await;
    assert_eq!(resp.status(), 200);

    let at_root = waste_of(&app, root()).await;
    assert_eq!(at_root["full_threshold"]["value"], 60);
    assert_eq!(at_root["full_threshold"]["inherited_value"], 80);
    let below = waste_of(&app, leaf).await;
    assert_eq!(below["full_threshold"]["value"], 60);
    assert_eq!(below["full_threshold"]["inherited_value"], 60);
}

#[tokio::test]
async fn sending_the_inherited_value_falls_back_and_is_recorded() {
    let app = waste_app().await;
    let org = child_org(&app, "Zurück", root()).await;

    app.put_json(&url(org), &json!({ "full_threshold": 60 }))
        .await;
    let resp = app
        .put_json(&url(org), &json!({ "full_threshold": 80 }))
        .await;
    assert_eq!(resp.status(), 200);

    let stored = sqlx::query_scalar!(
        "SELECT full_threshold_percent FROM waste_settings WHERE organization_id = $1",
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored, None, "no own value is kept");

    let last = sqlx::query!(
        r#"SELECT previous_value, new_value FROM organization_settings_history
           WHERE organization_id = $1 AND setting_key = 'full_threshold'
           ORDER BY changed_at DESC, id DESC LIMIT 1"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(last.previous_value, Some(json!(60)));
    assert_eq!(last.new_value, None);
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
