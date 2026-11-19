use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app},
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
};

async fn settings_of(app: &TestApp, org: Uuid) -> serde_json::Value {
    let resp = app
        .get(&format!("/api/v1/organizations/{org}/settings"))
        .await;
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

#[tokio::test]
async fn an_untouched_organization_reports_the_instance_default_twice() {
    let app = spawn_app().await;
    let org = child_org(&app, "Neu", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let body = settings_of(&app, org).await;

    let field = body["sensor_offline_after_secs"].as_object().unwrap();
    assert_eq!(field["value"], 86_400);
    assert_eq!(field["inherited_value"], 86_400);
    assert_eq!(field.len(), 2, "a field carries nothing but the two values");
    let top = body.as_object().unwrap();
    assert!(!top.contains_key("descendants_may_override"));
    assert!(!top.contains_key("enforced_by"));
}

#[tokio::test]
async fn a_value_set_above_is_the_inherited_value_below() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{root}/settings"),
            &json!({ "sensor_offline_after_secs": 100_000 }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, leaf).await;
    assert_eq!(body["sensor_offline_after_secs"]["value"], 100_000);
    assert_eq!(
        body["sensor_offline_after_secs"]["inherited_value"],
        100_000
    );
}

#[tokio::test]
async fn sending_the_inherited_value_makes_the_organization_follow_again() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    for (org, secs) in [(root, 100_000), (leaf, 70_000)] {
        app.put_json(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "sensor_offline_after_secs": secs }),
        )
        .await;
    }
    let body = settings_of(&app, leaf).await;
    assert_eq!(body["sensor_offline_after_secs"]["value"], 70_000);
    assert_eq!(
        body["sensor_offline_after_secs"]["inherited_value"],
        100_000
    );

    // The leaf types in what it would inherit anyway, then the root moves.
    for (org, secs) in [(leaf, 100_000), (root, 110_000)] {
        let resp = app
            .put_json(
                &format!("/api/v1/organizations/{org}/settings"),
                &json!({ "sensor_offline_after_secs": secs }),
            )
            .await;
        assert_eq!(resp.status(), 200);
    }

    assert_eq!(
        settings_of(&app, leaf).await["sensor_offline_after_secs"]["value"],
        110_000
    );
}

#[tokio::test]
async fn null_is_treated_like_an_omitted_field() {
    let app = spawn_app().await;
    let org = child_org(&app, "Null", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "sensor_offline_after_secs": 70_000 }),
    )
    .await;
    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "sensor_offline_after_secs": null }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    assert_eq!(
        settings_of(&app, org).await["sensor_offline_after_secs"]["value"],
        70_000
    );
}

#[tokio::test]
async fn an_omitted_field_is_left_alone() {
    let app = spawn_app().await;
    let org = child_org(&app, "Teilweise", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "sensor_offline_after_secs": 70_000, "defect_streak": 5 }),
    )
    .await;
    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "defect_streak": 7 }),
    )
    .await;

    let body = settings_of(&app, org).await;
    assert_eq!(body["sensor_offline_after_secs"]["value"], 70_000);
    assert_eq!(body["defect_streak"]["value"], 7);
}

#[tokio::test]
async fn an_out_of_range_value_answers_400_with_a_field_label() {
    let app = spawn_app().await;
    let org = child_org(&app, "Tippfehler", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "sensor_offline_after_secs": 3_000_000 }),
        )
        .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["validation"]["field"], "settings.sensor_offline_after");
    assert_eq!(
        body["validation"]["key"],
        "settings.sensor_offline_after.outOfRange"
    );
}

#[tokio::test]
async fn an_empty_write_leaves_the_organization_untouched() {
    let app = spawn_app().await;
    let org = child_org(&app, "Leer", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let resp = app
        .put_json(&format!("/api/v1/organizations/{org}/settings"), &json!({}))
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, org).await;
    assert_eq!(body["sensor_offline_after_secs"]["value"], 86_400);

    let stored = sqlx::query_scalar!(
        r#"SELECT count(*) FROM organization_settings WHERE organization_id = $1"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored, Some(0));

    let recorded = sqlx::query_scalar!(
        r#"SELECT count(*) FROM organization_settings_history WHERE organization_id = $1"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(recorded, Some(0));
}

/// The six map columns are the only positionally encoded value on this
/// endpoint, so a transposed argument anywhere between the DTO and the row
/// would be silent. The order has to survive a full write and read back.
#[tokio::test]
async fn a_map_view_round_trips_with_its_numbers_in_place() {
    let app = spawn_app().await;
    let org = child_org(
        &app,
        "Kartenausschnitt",
        Uuid::parse_str(ROOT_ORG_ID).unwrap(),
    )
    .await;

    let center = json!([54.79, 9.43]);
    let bbox = json!([54.71, 9.28, 54.86, 9.58]);

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "map_view": {
                "center": center, "bbox": bbox, "min_zoom": 13, "max_zoom": 18
            } }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, org).await;
    assert_eq!(body["map_view"]["value"]["center"], center);
    assert_eq!(body["map_view"]["value"]["bbox"], bbox);
    assert_eq!(body["map_view"]["value"]["min_zoom"], 13);
    assert_eq!(body["map_view"]["value"]["max_zoom"], 18);
    assert!(body["map_view"]["inherited_value"]["center"].is_array());

    let recorded = sqlx::query_scalar!(
        r#"SELECT new_value FROM organization_settings_history
           WHERE organization_id = $1 AND setting_key = 'map_view'"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(
        recorded,
        Some(json!({ "center": center, "bbox": bbox, "min_zoom": 13, "max_zoom": 18 }))
    );
}

#[tokio::test]
async fn organization_read_alone_does_not_permit_a_settings_read() {
    let (harness, app) = crate::auth_helpers::spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();

    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        root
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Nur Organisation lesen',
                   ARRAY['organization:read'])
           RETURNING id"#,
        root
    )
    .fetch_one(&app.db_pool)
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

    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    let resp = reqwest::Client::new()
        .get(format!(
            "{}/api/v1/organizations/{root}/settings",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn organization_update_alone_does_not_permit_a_settings_write() {
    let (harness, app) = crate::auth_helpers::spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();

    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        root
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Nur Organisation',
                   ARRAY['organization:read','organization:update'])
           RETURNING id"#,
        root
    )
    .fetch_one(&app.db_pool)
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

    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    let resp = reqwest::Client::new()
        .put(format!(
            "{}/api/v1/organizations/{root}/settings",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "sensor_offline_after_secs": 70_000 }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn setting_read_alone_does_not_permit_a_write_but_permits_a_read() {
    let (harness, app) = crate::auth_helpers::spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();

    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        root
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Nur lesen', ARRAY['setting:read'])
           RETURNING id"#,
        root
    )
    .fetch_one(&app.db_pool)
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

    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    let client = reqwest::Client::new();

    let read = client
        .get(format!(
            "{}/api/v1/organizations/{root}/settings",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(read.status(), 200);

    let write = client
        .put(format!(
            "{}/api/v1/organizations/{root}/settings",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "sensor_offline_after_secs": 70_000 }))
        .send()
        .await
        .unwrap();
    assert_eq!(write.status(), 403);
}

/// Without an existence check an unknown id resolves to a full set of instance
/// defaults and answers 200, which reads like a real organization.
#[tokio::test]
async fn an_unknown_organization_answers_404_on_both_verbs() {
    let app = spawn_app().await;
    let unknown = Uuid::now_v7();

    let resp = app
        .get(&format!("/api/v1/organizations/{unknown}/settings"))
        .await;
    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "resource.not_found");

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{unknown}/settings"),
            &json!({ "sensor_offline_after_secs": 70_000 }),
        )
        .await;
    assert_eq!(resp.status(), 404);
}
