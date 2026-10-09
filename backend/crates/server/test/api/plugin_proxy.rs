use serde_json::json;
use server::configuration::PluginProxySettings;

use crate::helpers::{TestApp, spawn_app_with_plugin_proxy, spawn_app_with_plugins};

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
