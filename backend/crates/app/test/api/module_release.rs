use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth_helpers::{AuthHarness, spawn_with_auth},
    helpers::{TestApp, seed_user_with_permissions, set_module, spawn_app},
    organizations::ROOT_ORG_ID,
};

const SCOPE_HEADER: &str = "X-Organization-Scope";

async fn create_tenant(app: &TestApp, name: &str) -> Uuid {
    let resp = app
        .post_json(
            "/api/v1/organizations",
            &json!({ "name": name, "parent_id": ROOT_ORG_ID }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().parse().unwrap()
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

async fn seed_root_member(harness: &AuthHarness, app: &TestApp, permissions: &[&str]) -> String {
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let root: Uuid = ROOT_ORG_ID.parse().unwrap();
    let role_id: Uuid = sqlx::query_scalar(
        "INSERT INTO roles (id, organization_id, name, permissions)
         VALUES (gen_random_uuid(), $1, 'Root-Rolle ' || gen_random_uuid()::text, $2) RETURNING id",
    )
    .bind(root)
    .bind(&permissions)
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(root)
        .execute(&app.db_pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&app.db_pool)
        .await
        .unwrap();
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

fn releases_path(org: impl std::fmt::Display) -> String {
    format!("/api/v1/organizations/{org}/module-releases")
}

fn modules_path(org: impl std::fmt::Display) -> String {
    format!("/api/v1/organizations/{org}/modules")
}

async fn release(app: &TestApp, tenant: Uuid, modules: Value) -> reqwest::Response {
    app.put_json(&releases_path(tenant), &json!({ "modules": modules }))
        .await
}

async fn get_json(app: &TestApp, path: &str) -> Value {
    let resp = app.get(path).await;
    assert_eq!(resp.status(), 200, "GET {path}");
    resp.json().await.unwrap()
}

fn module_of<'a>(list: &'a Value, key: &str) -> Option<&'a Value> {
    list["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == key)
}

fn tenant_entry(releases: &Value, tenant: Uuid) -> &Value {
    releases["tenants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["organization_id"] == tenant.to_string())
        .expect("tenant is listed")
}

#[tokio::test]
async fn a_new_tenant_has_no_module_released() {
    let app = spawn_app().await;
    app.without_release_fixture().await;
    let tenant = create_tenant(&app, "Neu").await;

    let releases = get_json(&app, "/api/v1/modules/releases").await;
    assert_eq!(releases["modules"], json!(["irrigation", "waste"]));
    let entry = tenant_entry(&releases, tenant);
    for key in ["irrigation", "waste"] {
        assert_eq!(module_of(entry, key).unwrap()["released"], false, "{key}");
    }

    let listed = get_json(&app, &modules_path(tenant)).await;
    assert_eq!(listed["modules"], json!([]));
}

#[tokio::test]
async fn the_first_release_switches_the_module_on() {
    let app = spawn_app().await;
    app.without_release_fixture().await;
    let tenant = create_tenant(&app, "Neu").await;

    let resp = release(&app, tenant, json!({ "waste": true })).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], tenant.to_string());
    let waste = module_of(&body, "waste").unwrap();
    assert_eq!(waste["released"], true);
    assert_eq!(waste["enabled"], true);

    let listed = get_json(&app, &modules_path(tenant)).await;
    let waste = module_of(&listed, "waste").unwrap();
    assert_eq!(waste["enabled"], true);
    assert_eq!(waste["switchable"], true);
}

#[tokio::test]
async fn locking_keeps_the_switch_and_a_new_release_restores_it() {
    let app = spawn_app().await;
    app.without_release_fixture().await;
    let tenant = create_tenant(&app, "Neu").await;

    assert_eq!(
        release(&app, tenant, json!({ "waste": true }))
            .await
            .status(),
        200
    );
    let resp = app
        .put_json(
            &modules_path(tenant),
            &json!({ "modules": { "waste": false } }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        release(&app, tenant, json!({ "waste": false }))
            .await
            .status(),
        200
    );
    let body: Value = release(&app, tenant, json!({ "waste": true }))
        .await
        .json()
        .await
        .unwrap();
    let waste = module_of(&body, "waste").unwrap();
    assert_eq!(waste["released"], true);
    assert_eq!(waste["enabled"], false);
}

#[tokio::test]
async fn a_locked_module_is_neither_listed_nor_switchable_for_the_tenant() {
    let (harness, app) = spawn_with_auth().await;
    let (tenant, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Mandant",
        &["setting:read", "setting:update"],
    )
    .await;
    set_module(&app, tenant, "waste", false, false).await;

    let body: Value = app
        .get_with_bearer("/api/v1/modules", &token)
        .await
        .json()
        .await
        .unwrap();
    assert!(
        !body["modules"]
            .as_array()
            .unwrap()
            .contains(&json!("waste"))
    );

    let resp = app.get_with_bearer(&modules_path(tenant), &token).await;
    assert_eq!(resp.status(), 200);
    let listed: Value = resp.json().await.unwrap();
    assert!(module_of(&listed, "waste").is_none());

    let resp = app
        .put_json_with_bearer(
            &modules_path(tenant),
            &json!({ "modules": { "waste": true } }),
            &token,
        )
        .await;
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.unknown");
}

#[tokio::test]
async fn a_sub_organization_follows_its_tenant_and_cannot_switch() {
    let app = spawn_app().await;
    let tenant = create_tenant(&app, "Mandant").await;
    let sub = child_of(&app, tenant, "Unter").await;

    let listed = get_json(&app, &modules_path(sub)).await;
    let irrigation = module_of(&listed, "irrigation").unwrap();
    assert_eq!(irrigation["enabled"], true);
    assert_eq!(irrigation["switchable"], false);

    let resp = app
        .put_json(
            &modules_path(sub),
            &json!({ "modules": { "irrigation": false } }),
        )
        .await;
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.not_switchable");

    let resp = app
        .put_json(
            &modules_path(tenant),
            &json!({ "modules": { "irrigation": false } }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let listed = get_json(&app, &modules_path(sub)).await;
    assert_eq!(module_of(&listed, "irrigation").unwrap()["enabled"], false);
}

#[tokio::test]
async fn only_members_of_the_root_release() {
    let (harness, app) = spawn_with_auth().await;
    let (tenant, tenant_admin) = seed_user_with_permissions(
        &harness,
        &app,
        "Mandant",
        &["setting:read", "setting:update"],
    )
    .await;
    let body = json!({ "modules": { "waste": true } });

    let resp = app
        .get_with_bearer("/api/v1/modules/releases", &tenant_admin)
        .await;
    assert_eq!(resp.status(), 403);
    let resp = app
        .put_json_with_bearer(&releases_path(tenant), &body, &tenant_admin)
        .await;
    assert_eq!(resp.status(), 403);

    let reader = seed_root_member(&harness, &app, &["setting:read"]).await;
    let resp = app
        .get_with_bearer("/api/v1/modules/releases", &reader)
        .await;
    assert_eq!(resp.status(), 200);
    let resp = app
        .put_json_with_bearer(&releases_path(tenant), &body, &reader)
        .await;
    assert_eq!(resp.status(), 403);

    let writer = seed_root_member(&harness, &app, &["setting:update"]).await;
    let resp = app
        .put_json_with_bearer(&releases_path(tenant), &body, &writer)
        .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn a_scoped_root_member_still_releases() {
    let (harness, app) = spawn_with_auth().await;
    let (tenant, _) = seed_user_with_permissions(&harness, &app, "Mandant", &[]).await;
    let member = seed_root_member(&harness, &app, &["setting:update"]).await;

    let resp = reqwest::Client::new()
        .put(format!("{}{}", app.address, releases_path(tenant)))
        .bearer_auth(&member)
        .header(SCOPE_HEADER, tenant.to_string())
        .json(&json!({ "modules": { "waste": true } }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn a_release_target_must_be_a_tenant() {
    let app = spawn_app().await;
    let tenant = create_tenant(&app, "Mandant").await;
    let sub = child_of(&app, tenant, "Unter").await;
    let body = json!({ "waste": true });

    for target in [ROOT_ORG_ID.parse::<Uuid>().unwrap(), sub] {
        let resp = release(&app, target, body.clone()).await;
        assert_eq!(resp.status(), 422, "{target}");
        let err: Value = resp.json().await.unwrap();
        assert_eq!(err["code"], "module.not_a_tenant");
    }

    let resp = release(&app, Uuid::now_v7(), body).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn releasing_an_unknown_module_is_rejected() {
    let app = spawn_app().await;
    let tenant = create_tenant(&app, "Mandant").await;
    let resp = release(&app, tenant, json!({ "nope": true })).await;
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.unknown");
}

#[tokio::test]
async fn locking_deletes_no_data() {
    let (harness, app) = spawn_with_auth().await;
    app.without_release_fixture().await;
    let root_member = seed_root_member(&harness, &app, &["setting:update"]).await;
    let (tenant, token) =
        seed_user_with_permissions(&harness, &app, "Mandant", &["waste_bin:read"]).await;
    let release = |released: bool| {
        let body = json!({ "modules": { "waste": released } });
        let path = releases_path(tenant);
        let (app, member) = (&app, &root_member);
        async move { app.put_json_with_bearer(&path, &body, member).await }
    };
    assert_eq!(release(true).await.status(), 200);
    sqlx::query(
        "INSERT INTO waste_bins (id, number, latitude, longitude, geometry, volume_liters, depth_cm, organization_id)
         VALUES ($1, 'LOCK-1', 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 1100, 120, $2)",
    )
    .bind(Uuid::now_v7())
    .bind(tenant)
    .execute(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(release(false).await.status(), 200);
    let locked = app.get_with_bearer("/api/v1/waste-bins", &token).await;
    assert_eq!(locked.status(), 403);
    let body: Value = locked.json().await.unwrap();
    assert_eq!(body["code"], "module.disabled");
    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM waste_bins WHERE organization_id = $1")
            .bind(tenant)
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(rows, 1);

    assert_eq!(release(true).await.status(), 200);
    let resp = app.get_with_bearer("/api/v1/waste-bins", &token).await;
    assert_eq!(resp.status(), 200);
    let bins: Value = resp.json().await.unwrap();
    let numbers: Vec<&str> = bins["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["number"].as_str().unwrap())
        .collect();
    assert!(numbers.contains(&"LOCK-1"), "{numbers:?}");
}

#[tokio::test]
async fn module_settings_stay_readable_while_released_but_off() {
    let (harness, app) = spawn_with_auth().await;
    let (tenant, token) =
        seed_user_with_permissions(&harness, &app, "Mandant", &["setting:read"]).await;
    let path = format!("/api/v1/organizations/{tenant}/settings/waste");

    set_module(&app, tenant, "waste", true, false).await;
    assert_eq!(app.get_with_bearer(&path, &token).await.status(), 200);

    set_module(&app, tenant, "waste", false, false).await;
    let resp = app.get_with_bearer(&path, &token).await;
    assert_eq!(resp.status(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.disabled");
}
