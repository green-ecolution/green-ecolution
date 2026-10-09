use std::net::SocketAddr;

use serde_json::json;
use server::configuration::PluginProxySettings;

use std::collections::BTreeSet;
use std::sync::Arc;
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

const PLUGIN_HOST_SUFFIX: &str = "plugins.test";

/// A client that resolves `<slug>.plugins.test` to the test server and never
/// follows redirects, so the proxy's own 303 and an upstream 302 stay visible.
fn plugin_client(app: &TestApp, slug: &str) -> (reqwest::Client, String) {
    let host = format!("{slug}.{PLUGIN_HOST_SUFFIX}");
    let client = reqwest::Client::builder()
        .resolve(&host, SocketAddr::from(([127, 0, 0, 1], app.port)))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    (client, format!("http://{host}:{}", app.port))
}

/// Opens a session the way the iframe does and returns the cookie pair.
async fn session_cookie(app: &TestApp, slug: &str) -> String {
    let (client, base) = plugin_client(app, slug);
    let ticket = view_ticket(app, slug).await;
    let resp = client
        .get(format!("{base}/__ge/session?ticket={ticket}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 303);
    assert_eq!(resp.headers()["location"], "/");
    let set_cookie = resp.headers()["set-cookie"].to_str().unwrap().to_string();
    set_cookie.split(';').next().unwrap().to_string()
}

async fn proxy_app_with(upstream: &MockServer) -> TestApp {
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![upstream.address().port()])).await;
    install_proxied(&app, "acme", upstream.address().port(), &[]).await;
    app
}

#[tokio::test]
async fn the_session_cookie_is_host_only_http_only_and_lax() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    let (client, base) = plugin_client(&app, "acme");
    let ticket = view_ticket(&app, "acme").await;

    let resp = client
        .get(format!("{base}/__ge/session?ticket={ticket}"))
        .send()
        .await
        .unwrap();

    let cookie = resp.headers()["set-cookie"].to_str().unwrap();
    assert!(cookie.starts_with("ge_plugin_session=gps_"), "{cookie}");
    assert!(
        cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"),
        "{cookie}"
    );
    assert!(!cookie.contains("Domain"), "{cookie}");
}

#[tokio::test]
async fn a_request_reaches_the_upstream_with_the_viewers_identity() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .and(header(
            "x-ge-user-id",
            "00000000-0000-0000-0000-000000000000",
        ))
        .and(header("x-ge-user-name", "ttester"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hi"))
        .expect(1)
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    let resp = client
        .get(format!("{base}/hello"))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    assert!(
        resp.headers()
            .get_all("content-security-policy")
            .iter()
            .any(|v| v
                .to_str()
                .unwrap()
                .contains("frame-ancestors http://127.0.0.1"))
    );
    assert_eq!(resp.text().await.unwrap(), "hi");
}

#[tokio::test]
async fn forged_identity_credentials_and_the_session_cookie_never_reach_the_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    client
        .get(format!("{base}/"))
        .header("cookie", format!("{cookie}; theme=dark"))
        .header("x-ge-user-id", "forged")
        .header("authorization", "Bearer stolen")
        .send()
        .await
        .unwrap();

    let received = upstream.received_requests().await.unwrap();
    let request = received.last().unwrap();
    assert_eq!(
        request.headers["x-ge-user-id"],
        "00000000-0000-0000-0000-000000000000"
    );
    assert!(request.headers.get("authorization").is_none());
    assert_eq!(request.headers["cookie"], "theme=dark");
}

#[tokio::test]
async fn a_request_without_a_session_is_refused() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    let (client, base) = plugin_client(&app, "acme");

    let resp = client.get(format!("{base}/")).send().await.unwrap();

    assert_eq!(resp.status().as_u16(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "plugin.proxy_session_invalid");
    assert!(upstream.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_reused_ticket_is_refused() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    let (client, base) = plugin_client(&app, "acme");
    let ticket = view_ticket(&app, "acme").await;
    let url = format!("{base}/__ge/session?ticket={ticket}");

    assert_eq!(
        client.get(&url).send().await.unwrap().status().as_u16(),
        303
    );
    assert_eq!(
        client.get(&url).send().await.unwrap().status().as_u16(),
        401
    );
}

#[tokio::test]
async fn a_session_of_one_plugin_does_not_open_another() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    install_proxied(&app, "other", upstream.address().port(), &[]).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "other");

    let resp = client
        .get(format!("{base}/"))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn opening_a_session_on_an_unknown_slug_is_not_found() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    let (client, base) = plugin_client(&app, "nobody");

    let resp = client
        .get(format!("{base}/__ge/session?ticket=gev_x"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn an_unreachable_upstream_is_a_bad_gateway() {
    // A dropped wiremock server returns to a pool and keeps listening, so a plain listener is used.
    let dead_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let app = spawn_app_with_plugin_proxy(proxy_settings(vec![dead_port])).await;
    install_proxied(&app, "acme", dead_port, &[]).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    let resp = client
        .get(format!("{base}/"))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 502);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "plugin.upstream_unreachable");
}

#[tokio::test]
async fn bodies_and_upstream_redirects_pass_through_unchanged() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/items"))
        .and(body_string("payload"))
        .respond_with(ResponseTemplate::new(201).set_body_string("created"))
        .mount(&upstream)
        .await;
    Mock::given(method("GET"))
        .and(path("/old"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/new"))
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    let created = client
        .post(format!("{base}/items"))
        .header("cookie", &cookie)
        .body("payload")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status().as_u16(), 201);
    assert_eq!(created.text().await.unwrap(), "created");

    let moved = client
        .get(format!("{base}/old"))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(moved.status().as_u16(), 302);
    assert_eq!(moved.headers()["location"], "/new");
}

#[tokio::test]
async fn the_view_of_a_proxied_plugin_carries_its_frontend_url() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;

    let view: serde_json::Value = app
        .get("/api/v1/plugins/acme/view")
        .await
        .json()
        .await
        .unwrap();
    let ticket = view["view_ticket"].as_str().unwrap();
    assert_eq!(
        view["frontend_url"],
        format!("http://acme.plugins.test/__ge/session?ticket={ticket}")
    );
}

#[tokio::test]
async fn the_api_still_answers_on_the_app_host() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    assert_eq!(app.get("/api/v1/plugins").await.status().as_u16(), 200);
}

#[tokio::test]
async fn a_preflight_on_a_plugin_host_reaches_the_upstream_without_the_apps_cors() {
    let upstream = MockServer::start().await;
    Mock::given(method("OPTIONS"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    let resp = client
        .request(reqwest::Method::OPTIONS, format!("{base}/items"))
        .header("cookie", &cookie)
        .header("origin", "http://somewhere.example")
        .header("access-control-request-method", "POST")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 204);
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn a_body_without_content_length_still_reaches_the_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/items"))
        .and(body_string("streamed"))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");
    let chunks = futures::stream::iter([Ok::<_, std::io::Error>("stream"), Ok("ed")]);

    let resp = client
        .post(format!("{base}/items"))
        .header("cookie", &cookie)
        .body(reqwest::Body::wrap_stream(chunks))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 201);
}

#[tokio::test]
async fn the_api_path_on_a_plugin_host_belongs_to_the_plugin() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/plugins"))
        .respond_with(ResponseTemplate::new(200).set_body_string("plugin"))
        .expect(1)
        .mount(&upstream)
        .await;
    let app = proxy_app_with(&upstream).await;
    let cookie = session_cookie(&app, "acme").await;
    let (client, base) = plugin_client(&app, "acme");

    let resp = client
        .get(format!("{base}/api/v1/plugins"))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.text().await.unwrap(), "plugin");
}

#[tokio::test]
async fn the_api_host_keeps_its_cors_with_the_proxy_on() {
    let upstream = MockServer::start().await;
    let app = proxy_app_with(&upstream).await;
    let client = reqwest::Client::new();

    let preflight = client
        .request(
            reqwest::Method::OPTIONS,
            format!("{}/api/v1/plugins", app.address),
        )
        .header("origin", "http://app.example.org")
        .header("access-control-request-method", "POST")
        .send()
        .await
        .unwrap();
    assert!(preflight.status().is_success());
    assert!(
        preflight
            .headers()
            .contains_key("access-control-allow-origin")
    );

    let missing = client
        .get(format!("{}/api/v1/nope", app.address))
        .header("origin", "http://app.example.org")
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status().as_u16(), 404);
    assert!(
        missing
            .headers()
            .contains_key("access-control-allow-origin")
    );
}
