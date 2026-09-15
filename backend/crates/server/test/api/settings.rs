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
async fn an_untouched_organization_reports_every_field_as_default() {
    let app = spawn_app().await;
    let org = child_org(&app, "Neu", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let body = settings_of(&app, org).await;

    assert_eq!(body["water_demand"]["origin"], "default");
    assert_eq!(body["water_demand"]["value"], 80.0);
    assert!(body["water_demand"]["source"].is_null());
    assert!(body["water_demand"]["own_value"].is_null());
    assert_eq!(body["descendants_may_override"], true);
    assert!(body["enforced_by"].is_null());
}

#[tokio::test]
async fn an_inherited_value_names_the_organization_it_comes_from() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{root}/settings"),
            &json!({ "water_demand": 100.0 }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, leaf).await;
    assert_eq!(body["water_demand"]["origin"], "inherited");
    assert_eq!(body["water_demand"]["value"], 100.0);
    assert_eq!(body["water_demand"]["source"]["id"], root.to_string());
}

#[tokio::test]
async fn null_restores_inheritance_and_the_next_read_says_so() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "water_demand": 100.0 }),
    )
    .await;
    app.put_json(
        &format!("/api/v1/organizations/{leaf}/settings"),
        &json!({ "water_demand": 70.0 }),
    )
    .await;
    assert_eq!(
        settings_of(&app, leaf).await["water_demand"]["origin"],
        "own"
    );

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{leaf}/settings"),
            &json!({ "water_demand": null }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, leaf).await;
    assert_eq!(body["water_demand"]["origin"], "inherited");
    assert_eq!(body["water_demand"]["value"], 100.0);
    assert!(body["water_demand"]["own_value"].is_null());
}

#[tokio::test]
async fn an_omitted_field_is_left_alone() {
    let app = spawn_app().await;
    let org = child_org(&app, "Teilweise", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "water_demand": 70.0, "defect_streak": 5 }),
    )
    .await;
    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "defect_streak": 7 }),
    )
    .await;

    let body = settings_of(&app, org).await;
    assert_eq!(body["water_demand"]["origin"], "own");
    assert_eq!(body["water_demand"]["value"], 70.0);
    assert_eq!(body["defect_streak"]["value"], 7);
}

#[tokio::test]
async fn a_dormant_own_value_is_still_reported() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Ruhend", root).await;

    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "water_demand": 100.0 }),
    )
    .await;
    app.put_json(
        &format!("/api/v1/organizations/{leaf}/settings"),
        &json!({ "water_demand": 70.0 }),
    )
    .await;
    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let body = settings_of(&app, leaf).await;
    assert_eq!(body["water_demand"]["value"], 100.0);
    assert_eq!(body["water_demand"]["own_value"], 70.0);
    assert_eq!(body["enforced_by"]["id"], root.to_string());
}

#[tokio::test]
async fn a_write_below_a_lock_answers_409_with_its_code() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Gesperrt", root).await;

    app.put_json(
        &format!("/api/v1/organizations/{root}/settings"),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{leaf}/settings"),
            &json!({ "water_demand": 70.0 }),
        )
        .await;
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn an_out_of_range_value_answers_400_with_a_field_label() {
    let app = spawn_app().await;
    let org = child_org(&app, "Tippfehler", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "water_demand": 100000.0 }),
        )
        .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["validation"]["field"], "settings.water_demand");
    assert_eq!(
        body["validation"]["key"],
        "settings.water_demand.outOfRange"
    );
}

#[tokio::test]
async fn the_last_change_is_reported_per_field() {
    let app = spawn_app().await;
    let org = child_org(&app, "Nachweis", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    app.put_json(
        &format!("/api/v1/organizations/{org}/settings"),
        &json!({ "water_demand": 70.0 }),
    )
    .await;

    let body = settings_of(&app, org).await;
    assert!(body["water_demand"]["last_change"]["changed_at"].is_string());
    assert!(body["defect_streak"]["last_change"].is_null());
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
    assert_eq!(body["water_demand"]["origin"], "default");
    assert_eq!(body["descendants_may_override"], true);

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
            &json!({ "map_view": { "center": center, "bbox": bbox } }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let body = settings_of(&app, org).await;
    assert_eq!(body["map_view"]["origin"], "own");
    assert_eq!(body["map_view"]["value"]["center"], center);
    assert_eq!(body["map_view"]["value"]["bbox"], bbox);
    assert_eq!(body["map_view"]["own_value"]["center"], center);
    assert_eq!(body["map_view"]["own_value"]["bbox"], bbox);

    let recorded = sqlx::query_scalar!(
        r#"SELECT new_value FROM organization_settings_history
           WHERE organization_id = $1 AND setting_key = 'map_view'"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(recorded, Some(json!({ "center": center, "bbox": bbox })));
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
        .json(&json!({ "water_demand": 70.0 }))
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
        .json(&json!({ "water_demand": 70.0 }))
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
            &json!({ "water_demand": 70.0 }),
        )
        .await;
    assert_eq!(resp.status(), 404);
}
