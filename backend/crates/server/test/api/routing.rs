use crate::helpers::{spawn_app, spawn_app_with_routing};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn encoded_line() -> String {
    let line = geo_types::LineString::from(vec![(9.4347, 54.7687), (9.4358, 54.7922)]);
    polyline::encode_coordinates(line, 6).unwrap()
}

fn streamlet_ok() -> serde_json::Value {
    serde_json::json!({
        "routes": [{
            "vehicle": 1,
            "stops": [
                {"VehicleStart": 1}, {"Refill": 1}, {"Customer": 1}, {"Depot": 1}
            ],
            "distance": 12500.0,
            "travel_time": 3600.0,
            "wait_time": 0.0,
            "geometry": {"format": "polyline", "value": encoded_line()}
        }],
        "unserved": [],
        "total_distance": 12500.0,
        "total_travel_time": 3600.0
    })
}

async fn mock_streamlet(response: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(response)
        .mount(&server)
        .await;
    server
}

async fn create_transporter(app: &crate::helpers::TestApp) -> serde_json::Value {
    let body = serde_json::json!({
        "number_plate": format!("FL RT {}", &uuid::Uuid::now_v7().simple().to_string()[..4]),
        "description": "Routing-Transporter",
        "water_capacity": 2000.0,
        "model": "Unimog",
        "status": "available",
        "type": "transporter",
        "driving_license": "BE",
        "height": 2.1, "width": 2.0, "length": 5.0, "weight": 3500.0
    });
    let resp = app.post_json("/api/v1/vehicles", &body).await;
    assert_eq!(resp.status().as_u16(), 201, "vehicle setup failed");
    resp.json().await.unwrap()
}

/// A cluster only gets coordinates (centroid) when it contains trees.
async fn create_cluster_with_tree(app: &crate::helpers::TestApp) -> String {
    let tree_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES ($1, 2020, 'Eiche', 'RT-001', 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'Routing')"#,
    )
    .bind(tree_id)
    .execute(&app.db_pool)
    .await
    .unwrap();
    let body = serde_json::json!({
        "name": "Routing-Cluster",
        "address": "Testweg 1",
        "description": "Routing",
        "soil_condition": "Su3",
        "tree_ids": [tree_id]
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(resp.status().as_u16(), 201, "cluster setup failed");
    let cluster: serde_json::Value = resp.json().await.unwrap();
    cluster["id"].as_str().unwrap().to_string()
}

fn plan_body(transporter_id: &str, cluster_id: &str) -> serde_json::Value {
    serde_json::json!({
        "date": "2026-08-15T06:00:00+00:00",
        "description": "Routing-Plan",
        "transporter_id": transporter_id,
        "tree_cluster_ids": [cluster_id],
        "user_ids": []
    })
}

#[tokio::test]
async fn create_plan_computes_route_metrics_and_geometry() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(plan["distance"], 12500.0);
    assert_eq!(plan["duration"], 3600.0);
    assert_eq!(plan["refill_count"], 1);
    assert_eq!(plan["total_water_required"], 80.0); // 1 tree × 80 l
    let plan_id = plan["id"].as_str().unwrap();
    assert_eq!(
        plan["gpx_url"],
        format!("/v1/watering-plans/{plan_id}/route/gpx")
    );

    let route_resp = app
        .get(&format!("/api/v1/watering-plans/{plan_id}/route"))
        .await;
    assert_eq!(route_resp.status().as_u16(), 200);
    let route: serde_json::Value = route_resp.json().await.unwrap();
    assert_eq!(route["geometry"]["type"], "LineString");
    let coords = route["geometry"]["coordinates"].as_array().unwrap();
    assert_eq!(coords.len(), 2);
    // GeoJSON position order: [lon, lat]
    assert!((coords[0][0].as_f64().unwrap() - 9.4347).abs() < 1e-4);
    assert!((coords[0][1].as_f64().unwrap() - 54.7687).abs() < 1e-4);
}

#[tokio::test]
async fn streamlet_failure_does_not_block_plan_creation() {
    let streamlet = mock_streamlet(ResponseTemplate::new(500)).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(plan["distance"], 0.0);
    assert_eq!(plan["gpx_url"], "");
}

#[tokio::test]
async fn route_endpoint_returns_404_without_route() {
    let streamlet = mock_streamlet(ResponseTemplate::new(500)).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let route_resp = app
        .get(&format!("/api/v1/watering-plans/{plan_id}/route"))
        .await;
    assert_eq!(route_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn preview_route_returns_route_response() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json(
            "/api/v1/watering-plans/route/preview",
            &serde_json::json!({ "cluster_ids": [cid], "transporter_id": tid }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let route: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(route["distance"], 12500.0);
    assert_eq!(route["geometry"]["type"], "LineString");
}

#[tokio::test]
async fn gpx_download_renders_track_from_geometry() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    let plan: serde_json::Value = resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let gpx_resp = app
        .get(&format!("/api/v1/watering-plans/{plan_id}/route/gpx"))
        .await;
    assert_eq!(gpx_resp.status().as_u16(), 200);
    assert_eq!(
        gpx_resp.headers()["content-type"].to_str().unwrap(),
        "application/gpx+xml"
    );
    let body = gpx_resp.text().await.unwrap();
    assert!(body.contains("<trkpt"));
    assert!(body.contains("green-ecolution"));
}

#[tokio::test]
async fn editing_plan_clears_stale_route_when_recompute_fails() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    // Create plan — first solve succeeds, plan gets route metrics.
    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(plan["distance"], 12500.0);
    let plan_id = plan["id"].as_str().unwrap();
    let route_resp = app
        .get(&format!("/api/v1/watering-plans/{plan_id}/route"))
        .await;
    assert_eq!(route_resp.status().as_u16(), 200);

    // Make the next solve call fail.
    streamlet.reset().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&streamlet)
        .await;

    // Edit the plan — recompute fails, so stale route must be cleared.
    let update_body = serde_json::json!({
        "date": "2026-08-15T06:00:00+00:00",
        "description": "Edited",
        "status": "planned",
        "transporter_id": tid,
        "tree_cluster_ids": [cid],
        "user_ids": [],
        "cancellation_note": ""
    });
    let update_resp = app
        .put_json(&format!("/api/v1/watering-plans/{plan_id}"), &update_body)
        .await;
    assert_eq!(update_resp.status().as_u16(), 200);
    let updated: serde_json::Value = update_resp.json().await.unwrap();
    assert_eq!(updated["distance"], 0.0, "stale distance must be cleared");

    // Route endpoint must return 404 — no geometry stored.
    let route_after = app
        .get(&format!("/api/v1/watering-plans/{plan_id}/route"))
        .await;
    assert_eq!(route_after.status().as_u16(), 404);
}

#[tokio::test]
async fn route_endpoint_returns_503_when_routing_disabled() {
    let app = spawn_app().await;

    let response = app
        .get("/api/v1/watering-plans/0190a8e9-7c4f-7000-8000-000000000000/route")
        .await;

    assert_eq!(response.status().as_u16(), 503);
    let body = response.text().await.unwrap_or_default();
    assert!(
        body.contains("routing"),
        "expected error body to mention routing, got: {body}"
    );
}
