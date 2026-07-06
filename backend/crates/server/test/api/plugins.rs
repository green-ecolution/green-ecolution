use crate::helpers::spawn_app;

async fn assert_plugins_503(response: reqwest::Response) {
    assert_eq!(
        response.status().as_u16(),
        503,
        "expected 503 Service Unavailable when plugins disabled"
    );
    let body = response.text().await.unwrap_or_default();
    assert!(
        body.contains("plugins"),
        "expected error body to mention plugins, got: {body}"
    );
}

#[tokio::test]
async fn list_plugins_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app.get("/api/v1/plugins").await;
    assert_plugins_503(response).await;
}

#[tokio::test]
async fn register_plugin_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app
        .post_json("/api/v1/plugins", &serde_json::json!({}))
        .await;
    assert_plugins_503(response).await;
}

#[tokio::test]
async fn get_plugin_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app.get("/api/v1/plugins/my-plugin").await;
    assert_plugins_503(response).await;
}

#[tokio::test]
async fn plugin_heartbeat_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app
        .post_json(
            "/api/v1/plugins/my-plugin/heartbeat",
            &serde_json::json!({}),
        )
        .await;
    assert_plugins_503(response).await;
}

#[tokio::test]
async fn plugin_refresh_token_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app
        .post_json(
            "/api/v1/plugins/my-plugin/token/refresh",
            &serde_json::json!({}),
        )
        .await;
    assert_plugins_503(response).await;
}

#[tokio::test]
async fn unregister_plugin_returns_503_when_disabled() {
    let app = spawn_app().await;
    let response = app
        .post_json(
            "/api/v1/plugins/my-plugin/unregister",
            &serde_json::json!({}),
        )
        .await;
    assert_plugins_503(response).await;
}
