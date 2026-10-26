use crate::helpers::{spawn_app, spawn_app_with_exact_modules};

fn module_count(body: &serde_json::Value, kind: &str) -> Option<i64> {
    body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["module"] == "irrigation" && m["kind"] == kind)
        .map(|m| m["count"].as_i64().unwrap())
}

#[tokio::test]
async fn statistics_returns_zero_for_empty_database() {
    let app = spawn_app().await;

    let body: serde_json::Value = app
        .get("/api/v1/info/statistics")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["sensorCount"], 0);
    assert_eq!(body["vehicleCount"], 0);
    assert_eq!(body["planCount"], 0);
    assert_eq!(module_count(&body, "tree"), Some(0));
    assert_eq!(module_count(&body, "cluster"), Some(0));
}

#[tokio::test]
async fn statistics_report_module_counts_for_trees_and_clusters() {
    let app = spawn_app().await;
    for i in 1..=2 {
        let tree = serde_json::json!({
            "species": "Eiche",
            "number": format!("T-STAT-{i}"),
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "Test"
        });
        app.post_json("/api/v1/trees", &tree).await;
    }
    let cluster = serde_json::json!({
        "name": "Cluster 1",
        "address": "Test",
        "description": "Test",
        "soil_condition": "Su3",
        "tree_ids": []
    });
    app.post_json("/api/v1/clusters", &cluster).await;

    let body: serde_json::Value = app
        .get("/api/v1/info/statistics")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(module_count(&body, "tree"), Some(2));
    assert_eq!(module_count(&body, "cluster"), Some(1));
}

#[tokio::test]
async fn statistics_without_modules_list_only_core_counts() {
    let app = spawn_app_with_exact_modules(vec![]).await;

    let body: serde_json::Value = app
        .get("/api/v1/info/statistics")
        .await
        .json()
        .await
        .unwrap();

    assert!(body["modules"].as_array().unwrap().is_empty());
    assert_eq!(body["sensorCount"], 0);
    assert_eq!(body["vehicleCount"], 0);
    assert_eq!(body["planCount"], 0);
}
