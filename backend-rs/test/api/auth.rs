use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

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

    // Different harness ⇒ different keys ⇒ invalid signature for this app.
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
async fn role_gated_create_user_returns_403_without_required_role() {
    let (harness, app) = spawn_with_auth().await;
    // Token only carries `smarte-grenzregion` — neither `tbz` nor `green-ecolution`.
    let token = harness.sign_token(json!({
        "realm_access": { "roles": ["smarte-grenzregion"] },
    }));

    let response = reqwest::Client::new()
        .post(format!("{}/api/v1/users", app.address))
        .bearer_auth(token)
        .json(&json!({
            "username": "new-user",
            "first_name": "New",
            "last_name": "User",
            "email": "new@example.com",
            "password": "S3cret!",
            "roles": ["tbz"],
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_url_handler_returns_keycloak_redirect() {
    let (harness, app) = spawn_with_auth().await;

    let response = app
        .get("/api/v1/users/login?redirect_url=http%3A%2F%2Flocalhost%2Fcb")
        .await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("json body");
    let login_url = body["login_url"].as_str().expect("login_url present");
    assert!(
        login_url.starts_with(&harness.issuer_url),
        "login_url should target the configured issuer ({}): got {login_url}",
        harness.issuer_url
    );
    assert!(login_url.contains("response_type=code"));
    assert!(login_url.contains("client_id=frontend"));
    assert!(login_url.contains("redirect_uri="));
}

#[tokio::test]
async fn auth_disabled_lets_protected_routes_through() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;
    let response = app.get("/api/v1/trees").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn auth_disabled_login_redirects_directly_to_callback_with_demo_code() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;

    let response = app
        .get("/api/v1/users/login?redirect_url=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fcallback%3Fredirect%3D%252Fdashboard")
        .await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    let login_url = body["login_url"].as_str().unwrap();
    assert!(
        login_url.starts_with("http://localhost:3000/auth/callback"),
        "should point at the SPA callback, got: {login_url}"
    );
    assert!(
        login_url.contains("code=demo"),
        "missing demo code: {login_url}"
    );
    assert!(
        login_url.contains("redirect=%2Fdashboard"),
        "should preserve existing query params: {login_url}"
    );
}

#[tokio::test]
async fn auth_disabled_exchange_code_returns_jwt_shaped_dummy_token() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;

    let response = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/users/login/token?redirect_url=http%3A%2F%2Flocalhost%2Fcb",
            app.address
        ))
        .json(&serde_json::json!({ "code": "demo" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    let access = body["access_token"].as_str().unwrap();
    let refresh = body["refresh_token"].as_str().unwrap();
    // JWT-shaped so the frontend's decodeJWT can read claims from the middle segment.
    assert_eq!(access.split('.').count(), 3);
    assert_eq!(refresh.split('.').count(), 3);
    assert_eq!(body["token_type"].as_str(), Some("Bearer"));
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

#[tokio::test]
async fn auth_disabled_logout_succeeds_without_calling_keycloak() {
    use crate::helpers::disabled_auth_settings;
    let app = spawn_app_with_auth(disabled_auth_settings()).await;

    let response = reqwest::Client::new()
        .post(format!("{}/api/v1/users/logout", app.address))
        .json(&serde_json::json!({ "refresh_token": "any.dummy.token" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
}
