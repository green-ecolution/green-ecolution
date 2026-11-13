use crate::helpers::{spawn_app, spawn_app_with_routing};
use crate::routing::{
    create_named_cluster_with_tree, create_transporter, mock_streamlet, plan_body, streamlet_ok,
};
use wiremock::ResponseTemplate;

#[tokio::test]
async fn itinerary_lists_depot_stop_and_station_in_driving_order() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let cluster = create_named_cluster_with_tree(&app, "Cluster Nord").await;
    let resp = app
        .post_json(
            "/api/v1/plans",
            &plan_body(transporter["id"].as_str().unwrap(), &cluster),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();

    let resp = app
        .get(&format!(
            "/api/v1/plans/{}/itinerary",
            plan["id"].as_str().unwrap()
        ))
        .await;

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["module"], "irrigation");
    assert_eq!(body["capacity_kind"], "water");
    let entries = body["entries"].as_array().unwrap();
    let kinds: Vec<&str> = entries
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["depot", "station", "stop", "depot"]);
    let indexes: Vec<u64> = entries
        .iter()
        .map(|e| e["index"].as_u64().unwrap())
        .collect();
    assert_eq!(indexes, (0..entries.len() as u64).collect::<Vec<_>>());
    let stop = entries.iter().find(|e| e["kind"] == "stop").unwrap();
    assert_eq!(stop["stop"], cluster.as_str());
    assert_eq!(stop["name"], "Cluster Nord");
    assert!(stop["lat"].is_number());
    assert!(stop["demand"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn itinerary_is_503_when_routing_is_disabled() {
    let app = spawn_app().await;
    let resp = app
        .get(&format!("/api/v1/plans/{}/itinerary", uuid::Uuid::now_v7()))
        .await;
    assert_eq!(resp.status().as_u16(), 503);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "feature.routing_disabled");
}

#[tokio::test]
async fn itinerary_of_unknown_plan_is_404() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let resp = app
        .get(&format!("/api/v1/plans/{}/itinerary", uuid::Uuid::now_v7()))
        .await;
    assert_eq!(resp.status().as_u16(), 404);
}
