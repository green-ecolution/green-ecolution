use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::spawn_app_with_auth;

#[tokio::test]
async fn public_info_route_does_not_require_auth() {
    let (_harness, app) = spawn_with_auth().await;

    let response = app.get("/api/v1/info").await;
    assert!(
        response.status().is_success(),
        "expected /info to be public, got {}",
        response.status()
    );
}

#[tokio::test]
async fn protected_route_rejects_missing_bearer() {
    let (_harness, app) = spawn_with_auth().await;

    let response = app.get("/api/v1/trees").await;
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_rejects_invalid_signature() {
    let (_harness, app) = spawn_with_auth().await;

    let other = AuthHarness::start().await;
    let token = other.sign_token(Value::Null);

    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/trees", app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_rejects_expired_token() {
    let (harness, app) = spawn_with_auth().await;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let token = harness.sign_token(json!({
        "iat": now - 600,
        "exp": now - 60,
    }));

    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/trees", app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_accepts_valid_token() {
    let (harness, app) = spawn_with_auth().await;
    let token = harness.sign_token(Value::Null);

    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/trees", app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("request");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "body: {:?}",
        response.text().await.ok()
    );
}

#[tokio::test]
async fn create_user_without_any_granted_role_returns_403() {
    let (harness, app) = spawn_with_auth().await;
    let token = harness.sign_token(Value::Null);

    let response = reqwest::Client::new()
        .post(format!("{}/api/v1/users", app.address))
        .bearer_auth(token)
        .json(&json!({
            "username": "new-user",
            "first_name": "New",
            "last_name": "User",
            "email": "new@example.com",
            "password": "S3cret!",
            "organization_id": Uuid::new_v4(),
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn auth_disabled_lets_protected_routes_through() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;
    let response = app.get("/api/v1/trees").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn auth_disabled_user_list_returns_demo_user_without_keycloak() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;

    let response = app.get("/api/v1/users").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    let items = body["data"].as_array().expect("data array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["username"].as_str(), Some("ttester"));
    assert_eq!(
        items[0]["email"].as_str(),
        Some("toni.tester@green-ecolution.de")
    );
}
