use std::collections::BTreeSet;

use domain::plugin::{PluginDraft, PluginFrontend, PluginName, PluginSlug, PluginWriter};
use server::infra::pg_plugin::PgPluginRepository;
use uuid::Uuid;

use crate::auth_helpers::AuthHarness;
use crate::helpers::{
    seed_user_with_permissions, spawn_app, spawn_app_with_plugins, spawn_app_with_plugins_and_auth,
    spawn_app_with_plugins_and_base_url,
};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const OTHER_ORG: &str = "01980000-0000-7000-8000-000000000003";

async fn assert_plugins_503(response: reqwest::Response) {
    assert_eq!(response.status().as_u16(), 503);
    let body = response.text().await.unwrap_or_default();
    assert!(
        body.contains("plugins"),
        "expected the error body to mention plugins, got: {body}"
    );
}

#[tokio::test]
async fn list_plugins_returns_503_when_disabled() {
    let app = spawn_app().await;
    assert_plugins_503(app.get("/api/v1/plugins").await).await;
}

#[tokio::test]
async fn ingest_returns_503_when_disabled() {
    let app = spawn_app().await;
    assert_plugins_503(
        app.post_json(
            "/api/v1/plugins/ingest/trees",
            &serde_json::json!({ "items": [] }),
        )
        .await,
    )
    .await;
}

#[tokio::test]
async fn install_returns_the_key_exactly_once() {
    let app = spawn_app_with_plugins().await;
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme",
                "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": ["tree:create"],
                "required_permissions": ["tree:read"],
                "frontend": { "mode": "none" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["key"].as_str().unwrap().starts_with("gep_"));

    let listed: serde_json::Value = app.get("/api/v1/plugins").await.json().await.unwrap();
    let entry = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["slug"] == "acme")
        .unwrap();
    assert!(
        entry.get("key").is_none(),
        "the key must never appear again"
    );
    assert!(
        entry.get("key_hash").is_none(),
        "the hash must never leave the server"
    );
}

#[tokio::test]
async fn slug_is_unique() {
    let app = spawn_app_with_plugins().await;
    let payload = serde_json::json!({
        "slug": "acme", "name": "Acme",
        "organization_id": "01980000-0000-7000-8000-000000000001",
        "permissions": [], "required_permissions": [], "frontend": { "mode": "none" }
    });
    assert_eq!(
        app.post_json("/api/v1/plugins", &payload)
            .await
            .status()
            .as_u16(),
        201
    );
    assert_eq!(
        app.post_json("/api/v1/plugins", &payload)
            .await
            .status()
            .as_u16(),
        409
    );
}

#[tokio::test]
async fn rotate_key_returns_a_different_key() {
    let app = spawn_app_with_plugins().await;
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme", "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [], "frontend": { "mode": "none" }
            }),
        )
        .await
        .json()
        .await
        .unwrap();

    let rotated: serde_json::Value = app
        .post_json("/api/v1/plugins/acme/key", &serde_json::json!({}))
        .await
        .json()
        .await
        .unwrap();
    assert_ne!(created["key"], rotated["key"]);
}

#[tokio::test]
async fn external_frontend_must_be_https() {
    let app = spawn_app_with_plugins().await;
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme", "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [],
                "frontend": { "mode": "external", "target": "http://plugin.example.com" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

/// `/plugins/me` is the ingest self-lookup route and matches before the
/// dynamic `/plugins/{plugin_slug}`, so a plugin installed under that slug
/// would answer 401 on GET and 405 on PATCH and DELETE forever, while still
/// holding its organization against deletion.
#[tokio::test]
async fn install_rejects_a_slug_reserved_by_the_plugin_routes() {
    let app = spawn_app_with_plugins().await;
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "me", "name": "Me",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [],
                "frontend": { "mode": "none" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

/// The iframe sandbox's `allow-same-origin` flag is safe only because a
/// plugin's frontend is served from a different origin than the app; without
/// this check an admin could point `frontend_target` at the app's own origin
/// and turn that flag into a full same-origin isolation failure.
#[tokio::test]
async fn external_frontend_must_not_be_the_apps_own_origin() {
    let app = spawn_app_with_plugins_and_base_url("https://app.example.com").await;
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme", "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [],
                "frontend": { "mode": "external", "target": "https://app.example.com" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

/// The same rule has to hold on the update path: installing with a legitimate
/// external target and afterwards patching it to the app's own origin would
/// otherwise reach the renderer with `allow-same-origin` intact.
#[tokio::test]
async fn external_frontend_must_not_become_the_apps_own_origin_on_update() {
    let app = spawn_app_with_plugins_and_base_url("https://app.example.com").await;
    let created = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme", "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [],
                "frontend": { "mode": "external", "target": "https://plugin.example.com" }
            }),
        )
        .await;
    assert_eq!(created.status().as_u16(), 201);

    let resp = app
        .patch_json(
            "/api/v1/plugins/acme",
            &serde_json::json!({
                "frontend": { "mode": "external", "target": "https://app.example.com" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);

    let stored: serde_json::Value = app
        .get("/api/v1/plugins/acme")
        .await
        .json()
        .await
        .expect("plugin body");
    assert_eq!(stored["frontend_target"], "https://plugin.example.com/");
}

/// Same host as the app's own origin, but a different port: this must be
/// accepted, proving the check compares scheme, host *and* port rather than
/// rejecting on a host substring match.
#[tokio::test]
async fn external_frontend_on_a_different_port_of_the_apps_host_is_allowed() {
    let app = spawn_app_with_plugins_and_base_url("https://app.example.com").await;
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &serde_json::json!({
                "slug": "acme", "name": "Acme",
                "organization_id": "01980000-0000-7000-8000-000000000001",
                "permissions": [], "required_permissions": [],
                "frontend": { "mode": "external", "target": "https://app.example.com:8443" }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
}

/// `PluginService::by_slug` (unlike every other plugin service method) does
/// not check authorization itself, since the ingest path authenticates via
/// the plugin's own key rather than a user token. The `GET /plugins/{slug}`
/// handler must therefore check `plugin:read` in the plugin's own
/// organization itself, or a caller who merely knows a slug could read a
/// plugin belonging to another organization.
#[tokio::test]
async fn get_plugin_is_forbidden_across_organizations() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugins_and_auth(harness.auth_settings(true)).await;

    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'Other Org')",
        Uuid::parse_str(OTHER_ORG).unwrap(),
        Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let repo = PgPluginRepository::new(app.db_pool.clone());
    repo.save_new(
        domain::Id::new_v7(),
        PluginDraft {
            slug: PluginSlug::new("other-org-plugin").unwrap(),
            name: PluginName::new("Other Org Plugin").unwrap(),
            description: None,
            organization_id: domain::Id::new(Uuid::parse_str(OTHER_ORG).unwrap()),
            permissions: BTreeSet::new(),
            required_permissions: BTreeSet::new(),
            frontend: PluginFrontend::None,
        },
        None,
    )
    .await
    .unwrap();

    let (_org_id, token) =
        seed_user_with_permissions(&harness, &app, "Caller Org", &["plugin:read"]).await;

    let resp = app
        .get_with_bearer("/api/v1/plugins/other-org-plugin", &token)
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}

/// Installs an enabled plugin with a frontend in `org`, bypassing the HTTP
/// surface so the test does not need an actor holding `plugin:create`.
async fn seed_plugin_with_view(
    app: &crate::helpers::TestApp,
    slug: &str,
    org: Uuid,
    required_permissions: &[&str],
) {
    let repo = PgPluginRepository::new(app.db_pool.clone());
    repo.save_new(
        domain::Id::new_v7(),
        PluginDraft {
            slug: PluginSlug::new(slug).unwrap(),
            name: PluginName::new("Demo Plugin").unwrap(),
            description: None,
            organization_id: domain::Id::new(org),
            permissions: BTreeSet::new(),
            required_permissions: required_permissions
                .iter()
                .map(|p| p.parse().unwrap())
                .collect(),
            frontend: PluginFrontend::External("https://plugin.example.org/view".parse().unwrap()),
        },
        None,
    )
    .await
    .unwrap();
    sqlx::query("UPDATE plugins SET enabled = TRUE WHERE slug = $1")
        .bind(slug)
        .execute(&app.db_pool)
        .await
        .unwrap();
}

/// The point of the split: the people a plugin view is built for hold the
/// plugin's `required_permissions`, not `plugin:read`.
#[tokio::test]
async fn plugin_view_opens_without_plugin_read() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugins_and_auth(harness.auth_settings(true)).await;
    let (org_id, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Baumpflege Org",
        &["tree:create", "tree:update"],
    )
    .await;
    seed_plugin_with_view(&app, "demo-plugin", org_id, &["tree:create", "tree:update"]).await;

    let resp = app
        .get_with_bearer("/api/v1/plugins/demo-plugin/view", &token)
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["frontend_mode"], "external");
    assert_eq!(body["frontend_target"], "https://plugin.example.org/view");
    assert!(
        body.get("permissions").is_none() && body.get("has_credential").is_none(),
        "the view response must not carry administrative facts: {body}"
    );

    // The management endpoint stays behind plugin:read, which this user lacks.
    let managed = app
        .get_with_bearer("/api/v1/plugins/demo-plugin", &token)
        .await;
    assert_eq!(managed.status().as_u16(), 403);
}

/// Every required permission has to be held; one of them is not enough.
#[tokio::test]
async fn plugin_view_requires_all_required_permissions() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugins_and_auth(harness.auth_settings(true)).await;
    let (org_id, token) =
        seed_user_with_permissions(&harness, &app, "Halbe Rechte Org", &["tree:create"]).await;
    seed_plugin_with_view(&app, "demo-plugin", org_id, &["tree:create", "tree:update"]).await;

    let resp = app
        .get_with_bearer("/api/v1/plugins/demo-plugin/view", &token)
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[tokio::test]
async fn plugin_view_is_refused_while_the_plugin_is_disabled() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugins_and_auth(harness.auth_settings(true)).await;
    let (org_id, token) =
        seed_user_with_permissions(&harness, &app, "Abgeschaltet Org", &["tree:create"]).await;
    seed_plugin_with_view(&app, "demo-plugin", org_id, &["tree:create"]).await;
    sqlx::query("UPDATE plugins SET enabled = FALSE WHERE slug = 'demo-plugin'")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let resp = app
        .get_with_bearer("/api/v1/plugins/demo-plugin/view", &token)
        .await;
    assert_eq!(resp.status().as_u16(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "plugin.disabled");
}

/// A plugin in another organization stays invisible even to someone holding
/// its required permissions in their own organization.
#[tokio::test]
async fn plugin_view_is_forbidden_across_organizations() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugins_and_auth(harness.auth_settings(true)).await;
    sqlx::query("INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'Other Org')")
        .bind(Uuid::parse_str(OTHER_ORG).unwrap())
        .bind(Uuid::parse_str(ROOT_ORG).unwrap())
        .execute(&app.db_pool)
        .await
        .unwrap();
    seed_plugin_with_view(
        &app,
        "other-org-view",
        Uuid::parse_str(OTHER_ORG).unwrap(),
        &["tree:create"],
    )
    .await;
    let (_org_id, token) =
        seed_user_with_permissions(&harness, &app, "Caller Org", &["tree:create"]).await;

    let resp = app
        .get_with_bearer("/api/v1/plugins/other-org-view/view", &token)
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}
