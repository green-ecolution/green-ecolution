use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::{
    auth_helpers::AuthHarness,
    helpers::{
        TestApp, spawn_app, spawn_app_with_data_export, spawn_app_with_data_export_and_auth,
    },
    organizations::ROOT_ORG_ID,
};

fn url(org: impl std::fmt::Display) -> String {
    format!("/api/v1/organizations/{org}/data-export")
}

fn root() -> Uuid {
    Uuid::parse_str(ROOT_ORG_ID).unwrap()
}

fn basic_target(base_url: &str, password: Option<&str>) -> Value {
    let mut auth = json!({ "kind": "basic", "username": "ge" });
    if let Some(p) = password {
        auth["password"] = json!(p);
    }
    json!({ "enabled": true, "base_url": base_url, "auth": auth })
}

async fn body(resp: reqwest::Response) -> Value {
    resp.json().await.unwrap()
}

#[tokio::test]
async fn the_endpoints_answer_503_while_the_instance_flag_is_off() {
    let app = spawn_app().await;

    let resp = app.get(&url(root())).await;

    assert_eq!(resp.status(), 503);
    assert_eq!(body(resp).await["code"], "feature.data_export_disabled");
}

#[tokio::test]
async fn a_missing_target_is_404() {
    let app = spawn_app_with_data_export(&["example.org"]).await;

    let resp = app.get(&url(root())).await;

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn put_stores_the_target_and_never_returns_the_secret() {
    let app = spawn_app_with_data_export(&["example.org"]).await;

    let resp = app
        .put_json(
            &url(root()),
            &basic_target(
                "https://frost.example.org/FROST-Server/v1.1",
                Some("geheim"),
            ),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(!text.contains("geheim"));

    let got = body(app.get(&url(root())).await).await;
    assert_eq!(
        got["base_url"],
        "https://frost.example.org/FROST-Server/v1.1"
    );
    assert_eq!(got["kind"], "sensorthings");
    assert_eq!(got["auth"]["kind"], "basic");
    assert_eq!(got["auth"]["username"], "ge");
    assert_eq!(got["auth"]["has_secret"], true);
    assert!(got["status"]["last_run_at"].is_null());
}

#[tokio::test]
async fn an_oauth2_target_round_trips_without_its_secret() {
    let app = spawn_app_with_data_export(&["example.org"]).await;

    let resp = app
        .put_json(
            &url(root()),
            &json!({ "enabled": true, "base_url": "https://frost.example.org/v1.1",
                     "auth": { "kind": "oauth2_client_credentials",
                               "token_url": "https://auth.example.org/token",
                               "client_id": "ge", "client_secret": "s", "scope": "export" } }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let auth = body(app.get(&url(root())).await).await["auth"].clone();
    assert_eq!(auth["kind"], "oauth2_client_credentials");
    assert_eq!(auth["client_id"], "ge");
    assert_eq!(auth["token_url"], "https://auth.example.org/token");
    assert_eq!(auth["scope"], "export");
    assert_eq!(auth["has_secret"], true);
    assert!(auth["username"].is_null());
    assert!(auth.get("client_secret").is_none());
}

#[tokio::test]
async fn a_resubmitted_form_without_secret_keeps_it() {
    let app = spawn_app_with_data_export(&["example.org"]).await;
    app.put_json(
        &url(root()),
        &basic_target("https://frost.example.org/v1.1", Some("pw")),
    )
    .await;

    let resp = app
        .put_json(
            &url(root()),
            &basic_target("https://frost.example.org/v1.1", None),
        )
        .await;

    assert_eq!(resp.status(), 200);
    assert_eq!(body(resp).await["auth"]["has_secret"], true);
}

#[tokio::test]
async fn a_stored_secret_does_not_follow_a_changed_host() {
    let app = spawn_app_with_data_export(&["example.org"]).await;
    app.put_json(
        &url(root()),
        &basic_target("https://frost.example.org/v1.1", Some("pw")),
    )
    .await;

    let resp = app
        .put_json(
            &url(root()),
            &basic_target("https://other.example.org/v1.1", None),
        )
        .await;

    assert_eq!(resp.status(), 422);
    assert_eq!(body(resp).await["code"], "data_export.auth_incomplete");
    let got = body(app.get(&url(root())).await).await;
    assert_eq!(got["base_url"], "https://frost.example.org/v1.1");
}

#[tokio::test]
async fn the_first_save_without_secret_is_422() {
    let app = spawn_app_with_data_export(&["example.org"]).await;

    let resp = app
        .put_json(
            &url(root()),
            &basic_target("https://frost.example.org/v1.1", None),
        )
        .await;

    assert_eq!(resp.status(), 422);
    assert_eq!(body(resp).await["code"], "data_export.auth_incomplete");
}

#[tokio::test]
async fn hosts_off_the_allowlist_are_refused() {
    let app = spawn_app_with_data_export(&["example.org"]).await;
    for base_url in [
        "https://db:5432/v1.1",
        "https://169.254.169.254/latest/meta-data",
        "https://example.org.evil.test/v1.1",
    ] {
        let resp = app
            .put_json(&url(root()), &basic_target(base_url, Some("pw")))
            .await;
        assert_eq!(resp.status(), 422, "{base_url}");
        assert_eq!(body(resp).await["code"], "data_export.url_not_allowed");
    }
}

#[tokio::test]
async fn malformed_and_overlong_input_is_400() {
    let app = spawn_app_with_data_export(&["example.org"]).await;
    let long = format!("https://frost.example.org/{}", "a".repeat(2100));
    for target in [
        basic_target("https://user:pw@frost.example.org/v1.1", Some("pw")),
        basic_target("not a url", Some("pw")),
        basic_target(&long, Some("pw")),
        json!({ "enabled": true, "base_url": "https://frost.example.org/v1.1",
                "auth": { "kind": "basic", "username": "x".repeat(300), "password": "pw" } }),
    ] {
        let resp = app.put_json(&url(root()), &target).await;
        assert_eq!(resp.status(), 400);
        assert_eq!(body(resp).await["code"], "request.validation_failed");
    }
}

#[tokio::test]
async fn delete_removes_the_target() {
    let app = spawn_app_with_data_export(&["example.org"]).await;
    app.put_json(
        &url(root()),
        &basic_target("https://frost.example.org/v1.1", Some("pw")),
    )
    .await;

    assert_eq!(app.delete(&url(root())).await.status(), 204);
    assert_eq!(app.get(&url(root())).await.status(), 404);
    assert_eq!(app.delete(&url(root())).await.status(), 404);
}

#[tokio::test]
async fn the_connection_test_reports_a_reachable_sensorthings_server() {
    let frost = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/FROST-Server/v1.1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "value": [],
            "serverSettings": { "conformance": [
                "http://www.opengis.net/spec/iot_sensing/1.1/req/data-array/data-array"
            ] }
        })))
        .mount(&frost)
        .await;
    let app = spawn_app_with_data_export(&["127.0.0.1:*"]).await;
    app.put_json(
        &url(root()),
        &json!({ "enabled": true, "base_url": format!("{}/FROST-Server/v1.1", frost.uri()),
                 "auth": { "kind": "none" } }),
    )
    .await;

    let resp = app
        .post_json(&format!("{}/test", url(root())), &json!({}))
        .await;

    assert_eq!(resp.status(), 200);
    let report = body(resp).await;
    assert_eq!(report["outcome"], "ok");
    assert_eq!(report["data_array_supported"], true);
}

async fn token_with(harness: &AuthHarness, app: &TestApp, permissions: &[&str]) -> String {
    let actor = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)",
        actor,
        root()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let perms: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, $2, $3) RETURNING id"#,
        root(),
        format!("Test {actor}"),
        &perms
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)",
        actor,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    harness.mock_identity_lookups(&[(actor, "ge.tester")]).await;
    harness.sign_token(json!({ "sub": actor.to_string() }))
}

#[tokio::test]
async fn reading_needs_setting_read_and_writing_setting_update() {
    let harness = AuthHarness::start().await;
    let app =
        spawn_app_with_data_export_and_auth(&["example.org"], harness.auth_settings(true)).await;
    let reader = token_with(&harness, &app, &["setting:read"]).await;
    let writer = token_with(&harness, &app, &["setting:read", "setting:update"]).await;
    let target = basic_target("https://frost.example.org/v1.1", Some("pw"));

    let denied = app
        .put_json_with_bearer(&url(root()), &target, &reader)
        .await;
    assert_eq!(denied.status(), 403);

    let allowed = app
        .put_json_with_bearer(&url(root()), &target, &writer)
        .await;
    assert_eq!(allowed.status(), 200);

    assert_eq!(
        app.get_with_bearer(&url(root()), &reader).await.status(),
        200
    );
    assert_eq!(
        app.post_json_with_bearer(&format!("{}/test", url(root())), &json!({}), &reader)
            .await
            .status(),
        403
    );
    assert_eq!(
        app.delete_with_bearer(&url(root()), &reader).await.status(),
        403
    );
}

#[tokio::test]
async fn a_sub_unit_below_a_settings_lock_cannot_set_up_its_own_target() {
    let app = spawn_app_with_data_export(&["frost.example.org"]).await;
    let child = crate::settings_repo::child_org(&app, "Gemeinde", root()).await;
    app.put_json(
        &format!("/api/v1/organizations/{}/settings", root()),
        &json!({ "descendants_may_override": false }),
    )
    .await;

    let resp = app
        .put_json(
            &url(child),
            &basic_target("https://frost.example.org/v1.1", Some("pw")),
        )
        .await;

    assert_eq!(resp.status(), 409);
    assert_eq!(body(resp).await["code"], "settings.enforced_by_ancestor");
}
