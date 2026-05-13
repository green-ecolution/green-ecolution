use std::collections::HashSet;

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
async fn get_info_returns_rust_version_not_go_version() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(body.get("rustVersion").is_some(), "rustVersion missing");
    assert!(
        body.get("goVersion").is_none(),
        "goVersion must not be present"
    );
    assert!(!body["rustVersion"].as_str().unwrap().is_empty());
    assert!(body["buildTime"].is_string());
}

#[tokio::test]
async fn get_server_returns_uptime_seconds_and_no_ip() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info/server").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(
        body["uptimeSeconds"].as_u64().is_some(),
        "uptimeSeconds must be a number"
    );
    assert!(
        body.get("uptime").is_none(),
        "old uptime string form must be gone"
    );
    assert!(body.get("ip").is_none(), "ip must not be present");
    assert!(body["url"].as_str().unwrap().starts_with("http"));
    assert!(body["port"].as_u64().is_some());
    assert!(body["interface"].is_string());
}

#[tokio::test]
async fn get_services_returns_expected_keys() {
    let app = spawn_app().await;

    // Test fixture sets health_check_interval_secs = 1; wait one tick + slack.
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    let response = app.get("/api/v1/info/services").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let items = body["items"].as_array().expect("items array");
    let names: HashSet<&str> = items.iter().map(|s| s["name"].as_str().unwrap()).collect();

    assert!(names.contains("database"), "database probe missing");
    assert!(names.contains("auth"), "auth probe missing");
    assert!(
        !names.contains("mqtt"),
        "mqtt should not appear when disabled"
    );

    for item in items {
        assert!(item["lastChecked"].is_string());
        assert!(item["responseTimeMs"].as_f64().is_some());
        assert!(item["message"].is_string());
    }
}

#[tokio::test]
async fn get_statistics_returns_counts() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info/statistics").await;
    let body: serde_json::Value = response.json().await.unwrap();

    for key in [
        "treeCount",
        "sensorCount",
        "vehicleCount",
        "treeClusterCount",
        "wateringPlanCount",
    ] {
        assert!(
            body[key].as_i64().is_some(),
            "{key} missing or not a number"
        );
    }
}
