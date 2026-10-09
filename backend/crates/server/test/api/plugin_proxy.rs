use serde_json::json;
use server::configuration::PluginProxySettings;

use std::collections::BTreeSet;
use std::sync::Arc;

use domain::plugin::{
    PluginDraft, PluginFrontend, PluginName, PluginSlug, PluginWriter, ServiceEndpoint,
};
use server::infra::pg_plugin::PgPluginRepository;
use server::service::ServiceError;
use server::service::plugin_proxy_service::PluginProxyService;

use crate::auth_helpers::AuthHarness;
use crate::helpers::{
    TestApp, seed_user_with_permissions_and_id, spawn_app_with_plugin_proxy,
    spawn_app_with_plugin_proxy_and_auth, spawn_app_with_plugins,
};

pub const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

pub fn proxy_settings(allowed_ports: Vec<u16>) -> PluginProxySettings {
    PluginProxySettings {
        public_url: "http://plugins.test".parse().unwrap(),
        allowed_service_suffixes: vec!["127.0.0.1".into()],
        allowed_ports,
        session_ttl_minutes: 480,
    }
}

async fn install_with_target(app: &TestApp, slug: &str, target: &str) -> reqwest::Response {
    app.post_json(
        "/api/v1/plugins",
        &json!({
            "slug": slug,
            "name": slug,
            "organization_id": ROOT_ORG,
            "permissions": [],
            "required_permissions": [],
            "frontend": { "mode": "proxied", "target": target }
        }),
    )
    .await
}

async fn code_of(response: reqwest::Response) -> String {
    let body: serde_json::Value = response.json().await.unwrap();
    body["code"].as_str().unwrap_or_default().to_string()
}

#[tokio::test]
async fn install_rejects_a_proxied_frontend_when_the_proxy_is_off() {
    let app = spawn_app_with_plugins().await;
    let resp = install_with_target(&app, "acme", "127.0.0.1:8080").await;
    assert_eq!(resp.status().as_u16(), 400);
    assert_eq!(code_of(resp).await, "plugin.proxy_unavailable");
}

#[tokio::test]
async fn install_rejects_a_proxied_target_outside_the_allowlist() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    let resp = install_with_target(&app, "acme", "metadata.google.internal:80").await;
    assert_eq!(resp.status().as_u16(), 400);
    assert_eq!(code_of(resp).await, "plugin.proxy_target_not_allowed");
}

#[tokio::test]
async fn install_accepts_an_allowlisted_proxied_target() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    let resp = install_with_target(&app, "acme", "127.0.0.1:8080").await;
    assert_eq!(resp.status().as_u16(), 201);
}

pub async fn install_proxied(app: &TestApp, slug: &str, port: u16, required: &[&str]) {
    let resp = app
        .post_json(
            "/api/v1/plugins",
            &json!({
                "slug": slug,
                "name": slug,
                "organization_id": ROOT_ORG,
                "permissions": [],
                "required_permissions": required,
                "frontend": { "mode": "proxied", "target": format!("127.0.0.1:{port}") }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201, "install {slug}");
    app.patch_json(
        &format!("/api/v1/plugins/{slug}"),
        &json!({ "enabled": true }),
    )
    .await;
}

pub async fn view_ticket(app: &TestApp, slug: &str) -> String {
    let view: serde_json::Value = app
        .get(&format!("/api/v1/plugins/{slug}/view"))
        .await
        .json()
        .await
        .unwrap();
    view["view_ticket"].as_str().unwrap().to_string()
}

fn proxy_service(app: &TestApp) -> Arc<PluginProxyService> {
    app.state
        .plugin_proxy_service
        .clone()
        .expect("proxy configured")
}

fn slug(s: &str) -> PluginSlug {
    PluginSlug::new(s).unwrap()
}

pub async fn seed_proxied_plugin(
    app: &TestApp,
    slug: &str,
    org: uuid::Uuid,
    port: u16,
    required: &[&str],
) {
    PgPluginRepository::new(app.db_pool.clone())
        .save_new(
            domain::Id::new_v7(),
            PluginDraft {
                slug: PluginSlug::new(slug).unwrap(),
                name: PluginName::new(slug).unwrap(),
                description: None,
                organization_id: domain::Id::new(org),
                permissions: BTreeSet::new(),
                required_permissions: required.iter().map(|p| p.parse().unwrap()).collect(),
                device_capabilities: BTreeSet::new(),
                frontend: PluginFrontend::Proxied(ServiceEndpoint::new("127.0.0.1", port).unwrap()),
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

#[tokio::test]
async fn an_opened_session_authorizes_with_endpoint_and_identity() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    install_proxied(&app, "acme", 8080, &[]).await;
    let svc = proxy_service(&app);

    let token = svc
        .open_session(&slug("acme"), &view_ticket(&app, "acme").await)
        .await
        .unwrap();
    assert!(token.starts_with("gps_"));

    let grant = svc.authorize(&slug("acme"), Some(&token)).await.unwrap();
    assert_eq!(grant.endpoint.host(), "127.0.0.1");
    assert_eq!(grant.endpoint.port(), 8080);
    assert_eq!(grant.user_id, uuid::Uuid::nil());
    assert_eq!(grant.user_display_name, "ttester");
}

#[tokio::test]
async fn a_ticket_opens_only_one_session() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    install_proxied(&app, "acme", 8080, &[]).await;
    let svc = proxy_service(&app);
    let ticket = view_ticket(&app, "acme").await;

    svc.open_session(&slug("acme"), &ticket).await.unwrap();
    let err = svc.open_session(&slug("acme"), &ticket).await.unwrap_err();
    assert_eq!(err.code(), "plugin.view_ticket_invalid");
}

#[tokio::test]
async fn a_missing_or_foreign_token_is_rejected() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    install_proxied(&app, "acme", 8080, &[]).await;
    install_proxied(&app, "other", 8080, &[]).await;
    let svc = proxy_service(&app);
    let token = svc
        .open_session(&slug("acme"), &view_ticket(&app, "acme").await)
        .await
        .unwrap();

    for (target, token) in [
        ("acme", None),
        ("acme", Some("gps_unknown")),
        ("other", Some(token.as_str())),
    ] {
        let err = svc.authorize(&slug(target), token).await.unwrap_err();
        assert_eq!(
            err.code(),
            "plugin.proxy_session_invalid",
            "{target} {token:?}"
        );
    }
}

#[tokio::test]
async fn disabling_the_plugin_ends_its_sessions() {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![8080])).await;
    install_proxied(&app, "acme", 8080, &[]).await;
    let svc = proxy_service(&app);
    let token = svc
        .open_session(&slug("acme"), &view_ticket(&app, "acme").await)
        .await
        .unwrap();

    app.patch_json("/api/v1/plugins/acme", &json!({ "enabled": false }))
        .await;

    let err = svc
        .authorize(&slug("acme"), Some(&token))
        .await
        .unwrap_err();
    assert!(matches!(err, ServiceError::Auth(_)));
    assert_eq!(err.code(), "plugin.disabled");
}

#[tokio::test]
async fn revoking_the_role_ends_the_session() {
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_plugin_proxy_and_auth(
        proxy_settings(vec![8080]),
        harness.auth_settings(true),
    )
    .await;
    let (org_id, token, user_id) =
        seed_user_with_permissions_and_id(&harness, &app, "Kataster Org", &["tree:read"]).await;
    seed_proxied_plugin(&app, "acme", org_id, 8080, &["tree:read"]).await;

    let view: serde_json::Value = app
        .get_with_bearer("/api/v1/plugins/acme/view", &token)
        .await
        .json()
        .await
        .unwrap();
    let svc = proxy_service(&app);
    let session = svc
        .open_session(&slug("acme"), view["view_ticket"].as_str().unwrap())
        .await
        .unwrap();
    assert!(svc.authorize(&slug("acme"), Some(&session)).await.is_ok());

    sqlx::query("DELETE FROM role_assignments WHERE user_id = $1")
        .bind(user_id)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let err = svc
        .authorize(&slug("acme"), Some(&session))
        .await
        .unwrap_err();
    assert_eq!(err.code(), "auth.forbidden");
}
