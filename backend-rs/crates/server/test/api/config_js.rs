use crate::helpers::spawn_app;

#[tokio::test]
async fn get_config_js_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/config.js").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn get_config_js_body_contains_window_env() {
    let app = spawn_app().await;

    let response = app.get("/api/config.js").await;
    let body = response.text().await.expect("failed to read body");

    assert!(body.contains("window._env_"));
}

#[tokio::test]
async fn get_config_js_content_type_is_javascript() {
    let app = spawn_app().await;

    let response = app.get("/api/config.js").await;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("javascript"));
}
