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

    let version = body["version"].as_str().expect("version is a string");
    // Test binary is compiled in debug mode → version must carry the +dev.{commit} suffix.
    assert!(
        version.starts_with(&format!("{}+dev.", env!("CARGO_PKG_VERSION"))),
        "expected +dev.<commit> suffix on debug build, got {version}"
    );
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
async fn get_info_returns_rust_metadata() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(
        body.get("goVersion").is_none(),
        "goVersion must not be present"
    );
    assert!(!body["rustVersion"].as_str().unwrap().is_empty());
    assert!(!body["rustChannel"].as_str().unwrap().is_empty());
    assert_eq!(body["rustEdition"], "2024");
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

    let response = app.get("/api/v1/info/services").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let items = body["items"].as_array().expect("items array");
    let names: HashSet<&str> = items.iter().map(|s| s["name"].as_str().unwrap()).collect();

    assert!(names.contains("database"), "database probe missing");
    assert!(names.contains("auth"), "auth probe missing");
    assert!(names.contains("mqtt"), "mqtt probe missing");

    let mqtt = items
        .iter()
        .find(|s| s["name"] == "mqtt")
        .expect("mqtt entry present");
    assert_eq!(mqtt["enabled"], false, "mqtt should report as disabled");
    assert_eq!(mqtt["message"], "service.status.disabled");

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

#[tokio::test]
async fn services_info_lists_routing_and_plugins_disabled_by_default() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/info/services").await;
    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.expect("json body");
    let items = body
        .get("items")
        .and_then(|v| v.as_array())
        .expect("items array");

    let by_name = |name: &str| -> &serde_json::Value {
        items
            .iter()
            .find(|i| i.get("name").and_then(|n| n.as_str()) == Some(name))
            .unwrap_or_else(|| panic!("missing service entry: {name}"))
    };

    let routing = by_name("routing");
    assert_eq!(
        routing.get("enabled").and_then(|v| v.as_bool()),
        Some(false)
    );

    let plugins = by_name("plugins");
    assert_eq!(
        plugins.get("enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
}
