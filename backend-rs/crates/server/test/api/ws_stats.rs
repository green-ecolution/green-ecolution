use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::spawn_app;

#[tokio::test]
async fn ws_stats_rejects_request_without_token() {
    let (_harness, app) = spawn_with_auth().await;
    let url = format!("{}/api/v1/ws/stats", app.ws_url());
    let result = tokio_tungstenite::connect_async(url).await;
    assert!(result.is_err(), "expected handshake failure without token");
}

#[tokio::test]
async fn ws_stats_rejects_invalid_token() {
    let (_harness, app) = spawn_with_auth().await;
    let url = format!("{}/api/v1/ws/stats?token=garbage", app.ws_url());
    let result = tokio_tungstenite::connect_async(url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn ws_stats_streams_payload_with_valid_token() {
    let (harness, app) = spawn_with_auth().await;
    let token = harness.sign_token(serde_json::json!({}));
    let url = format!("{}/api/v1/ws/stats?token={}", app.ws_url(), token);
    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("ws connect");

    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("timeout")
        .expect("stream ended")
        .expect("ws error");

    let text = match msg {
        Message::Text(t) => t.to_string(),
        other => panic!("expected text, got {other:?}"),
    };
    let payload: Value = serde_json::from_str(&text).expect("valid json");

    assert!(payload["memory"]["residentBytes"].as_u64().is_some());
    assert!(payload["memory"]["virtualBytes"].as_u64().is_some());
    assert!(payload["cpu"]["cores"].as_u64().is_some());
    assert!(payload["cpu"]["usagePercent"].as_f64().is_some());
    assert!(payload["tokio"]["workerThreads"].as_u64().is_some());
    assert!(payload["dbPool"]["max"].as_u64().is_some());
    assert!(payload["process"]["uptimeSeconds"].as_u64().is_some());
    assert!(payload["timestamp"].as_i64().is_some());
}

#[tokio::test]
async fn ws_stats_streams_without_token_when_auth_disabled() {
    let app = spawn_app().await;
    let url = format!("{}/api/v1/ws/stats", app.ws_url());
    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("ws connect (auth disabled)");

    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("timeout")
        .expect("stream ended")
        .expect("ws error");
    assert!(matches!(msg, Message::Text(_)));
}

#[tokio::test]
async fn ws_stats_closes_on_client_disconnect() {
    let app = spawn_app().await;
    let url = format!("{}/api/v1/ws/stats", app.ws_url());
    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("ws connect");
    ws.send(Message::Close(None)).await.unwrap();

    let result = tokio::time::timeout(Duration::from_secs(2), ws.next()).await;
    assert!(
        result.is_ok(),
        "server should drop the connection within 2s"
    );
}
