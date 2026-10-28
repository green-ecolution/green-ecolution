use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth_helpers::{AuthHarness, spawn_with_auth},
    helpers::TestApp,
    organizations::ROOT_ORG_ID,
};

async fn child_org(app: &TestApp, name: &str, parent: Uuid) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        parent,
        name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

async fn settings_admin_in(harness: &AuthHarness, app: &TestApp, org: Uuid) -> String {
    let permissions = vec!["setting:read".to_string(), "setting:update".to_string()];
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Test-Rolle', $2)
           RETURNING id"#,
        org,
        &permissions,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
    )
    .execute(&app.db_pool)
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
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

async fn switch(app: &TestApp, token: &str, org: Uuid, waste: Value) -> reqwest::Response {
    app.put_json_with_bearer(
        &format!("/api/v1/organizations/{org}/modules"),
        &json!({ "modules": { "waste": waste } }),
        token,
    )
    .await
}

async fn waste_row(app: &TestApp, token: &str, org: Uuid) -> Value {
    let resp = app
        .get_with_bearer(&format!("/api/v1/organizations/{org}/modules"), token)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == "waste")
        .cloned()
        .unwrap()
}

async fn assert_set_by_ancestor(resp: reqwest::Response, ancestor: Uuid) {
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "settings.set_by_ancestor");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains(&ancestor.to_string())
    );
}

struct Tree {
    parent: Uuid,
    child: Uuid,
    parent_token: String,
    child_token: String,
}

async fn parent_and_child(harness: &AuthHarness, app: &TestApp) -> Tree {
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let parent = child_org(app, "Eltern", root).await;
    let child = child_org(app, "Kind", parent).await;
    Tree {
        parent,
        child,
        parent_token: settings_admin_in(harness, app, parent).await,
        child_token: settings_admin_in(harness, app, child).await,
    }
}

#[tokio::test]
async fn a_switch_set_from_above_binds_the_sub_unit() {
    let (harness, app) = spawn_with_auth().await;
    let t = parent_and_child(&harness, &app).await;

    assert_eq!(
        switch(&app, &t.parent_token, t.parent, json!(true))
            .await
            .status(),
        200
    );
    assert_eq!(
        switch(&app, &t.parent_token, t.child, json!(false))
            .await
            .status(),
        200
    );

    let row = waste_row(&app, &t.child_token, t.child).await;
    assert_eq!(row["enabled"], false);
    assert_eq!(row["set_by"]["id"], t.parent.to_string());
    assert_eq!(row["fixed_by_ancestor"], true);

    assert_set_by_ancestor(
        switch(&app, &t.child_token, t.child, json!(true)).await,
        t.parent,
    )
    .await;
    assert_set_by_ancestor(
        switch(&app, &t.child_token, t.child, Value::Null).await,
        t.parent,
    )
    .await;

    let parent_view = waste_row(&app, &t.parent_token, t.child).await;
    assert_eq!(parent_view["fixed_by_ancestor"], false);
    assert_eq!(
        switch(&app, &t.parent_token, t.child, json!(true))
            .await
            .status(),
        200
    );
}

#[tokio::test]
async fn resending_the_imposed_value_changes_nothing() {
    let (harness, app) = spawn_with_auth().await;
    let t = parent_and_child(&harness, &app).await;

    switch(&app, &t.parent_token, t.child, json!(false)).await;
    assert_eq!(
        switch(&app, &t.child_token, t.child, json!(false))
            .await
            .status(),
        200
    );

    let row = waste_row(&app, &t.child_token, t.child).await;
    assert_eq!(row["set_by"]["id"], t.parent.to_string());
    assert_eq!(row["fixed_by_ancestor"], true);
}

#[tokio::test]
async fn a_sub_units_own_switch_stays_open_until_the_parent_overrides_it() {
    let (harness, app) = spawn_with_auth().await;
    let t = parent_and_child(&harness, &app).await;

    assert_eq!(
        switch(&app, &t.child_token, t.child, json!(true))
            .await
            .status(),
        200
    );
    let row = waste_row(&app, &t.child_token, t.child).await;
    assert_eq!(row["set_by"]["id"], t.child.to_string());
    assert_eq!(row["fixed_by_ancestor"], false);
    assert_eq!(
        switch(&app, &t.child_token, t.child, json!(false))
            .await
            .status(),
        200
    );

    assert_eq!(
        switch(&app, &t.parent_token, t.child, json!(true))
            .await
            .status(),
        200
    );
    assert_set_by_ancestor(
        switch(&app, &t.child_token, t.child, json!(false)).await,
        t.parent,
    )
    .await;
}

#[tokio::test]
async fn a_middle_level_cannot_change_what_a_higher_level_set_below_it() {
    let (harness, app) = spawn_with_auth().await;
    let t = parent_and_child(&harness, &app).await;
    let grandchild = child_org(&app, "Enkel", t.child).await;

    switch(&app, &t.parent_token, grandchild, json!(true)).await;

    assert_set_by_ancestor(
        switch(&app, &t.child_token, grandchild, json!(false)).await,
        t.parent,
    )
    .await;
    assert_eq!(
        waste_row(&app, &t.child_token, grandchild).await["fixed_by_ancestor"],
        true
    );
}
