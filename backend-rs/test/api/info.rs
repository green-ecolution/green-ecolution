use crate::helpers::spawn_app;

#[tokio::test]
async fn get_info_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn get_info_returns_version() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(body["version"].is_string());
    assert!(!body["version"].as_str().unwrap().is_empty());
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn get_info_returns_version_info() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let vi = &body["versionInfo"];
    assert!(vi["current"].is_string());
    assert!(vi.get("updateAvailable").is_some());
    assert!(vi.get("isDevelopment").is_some());
    assert!(vi["releaseUrl"].is_string());
}

#[tokio::test]
async fn get_info_returns_git_info() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let git = &body["git"];
    assert!(git["branch"].is_string());
    assert!(git["commit"].is_string());
    assert!(git["repository"].is_string());
}

#[tokio::test]
async fn get_info_returns_map_info() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let map = &body["map"];
    assert_eq!(map["center"].as_array().unwrap().len(), 2);
    assert_eq!(map["bbox"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn get_info_returns_rust_version_and_build_time() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(body["goVersion"].as_str().unwrap().contains("rustc"));
    assert!(body["buildTime"].is_string());
}
