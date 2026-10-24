use crate::helpers::{install_plugin, spawn_app_with_plugins};

#[tokio::test]
async fn unknown_key_is_rejected() {
    let app = spawn_app_with_plugins().await;
    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &serde_json::json!({ "items": [] }),
            "gep_01980000-0000-7000-8000-0000000000c1.deadbeef",
        )
        .await;
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn malformed_key_is_rejected() {
    let app = spawn_app_with_plugins().await;
    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &serde_json::json!({ "items": [] }),
            "not-a-plugin-key",
        )
        .await;
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn me_returns_the_authenticated_plugin() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "acme", &["tree:read"]).await;

    let body: serde_json::Value = app
        .get_with_bearer("/api/v1/plugins/me", &key)
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["slug"], "acme");
    assert!(body.get("key").is_none());
}

#[tokio::test]
async fn disabled_plugin_is_forbidden() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "acme", &["tree:read"]).await;
    app.patch_json(
        "/api/v1/plugins/acme",
        &serde_json::json!({ "enabled": false }),
    )
    .await;

    let resp = app.get_with_bearer("/api/v1/plugins/me", &key).await;
    assert_eq!(resp.status().as_u16(), 403);
}
