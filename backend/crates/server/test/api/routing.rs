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

    let refill_points = route["refill_points"].as_array().unwrap();
    assert_eq!(refill_points.len(), 1);
    assert_eq!(refill_points[0]["name"], "Betriebshof Schleswiger Straße");
    assert!((refill_points[0]["lat"].as_f64().unwrap() - 54.76879146396569).abs() < 1e-9);
    assert!((refill_points[0]["lon"].as_f64().unwrap() - 9.434803531218018).abs() < 1e-9);
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

    let refill_points = route["refill_points"].as_array().unwrap();
    assert_eq!(refill_points.len(), 1);
    assert_eq!(refill_points[0]["name"], "Betriebshof Schleswiger Straße");
}

#[tokio::test]
async fn route_without_refill_stops_has_empty_refill_points() {
    let mut body = streamlet_ok();
    body["routes"][0]["stops"] = serde_json::json!([
        {"VehicleStart": 1}, {"Customer": 1}, {"Depot": 1}
    ]);
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(body)).await;
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
    assert_eq!(route_resp.status().as_u16(), 200);
    let route: serde_json::Value = route_resp.json().await.unwrap();
    assert_eq!(route["refill_points"].as_array().unwrap().len(), 0);
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

#[tokio::test]
async fn start_points_endpoint_lists_configured_points() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let resp = app.get("/api/v1/routing/start-points").await;
    assert_eq!(resp.status().as_u16(), 200);
    let points: serde_json::Value = resp.json().await.unwrap();
    let arr = points.as_array().unwrap();
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["name"], "Betriebshof Schleswiger Straße");
}

#[tokio::test]
async fn start_points_endpoint_returns_503_when_disabled() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/routing/start-points").await;
    assert_eq!(resp.status().as_u16(), 503);
}

#[tokio::test]
async fn plan_with_start_point_name_routes_from_that_depot() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let mut body = plan_body(tid, &cid);
    body["start_point_name"] = serde_json::Value::String("Depot Nord".to_string());
    let resp = app.post_json("/api/v1/watering-plans", &body).await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(plan["start_point_name"], "Depot Nord");

    // Check that streamlet received a depot near Depot Nord (lat≈54.81)
    let requests = streamlet.received_requests().await.unwrap();
    let last = requests.last().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&last.body).unwrap();
    let depot_lat = body["problem"]["depots"][0]["location"]["lat"]
        .as_f64()
        .unwrap();
    assert!(
        (depot_lat - 54.81).abs() < 0.01,
        "expected depot lat≈54.81, got {depot_lat}"
    );
}

#[tokio::test]
async fn preview_with_start_point_name_uses_that_depot() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json(
            "/api/v1/watering-plans/route/preview",
            &serde_json::json!({
                "cluster_ids": [cid],
                "transporter_id": tid,
                "start_point_name": "Depot Nord"
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 200);

    let requests = streamlet.received_requests().await.unwrap();
    let last = requests.last().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&last.body).unwrap();
    let depot_lat = body["problem"]["depots"][0]["location"]["lat"]
        .as_f64()
        .unwrap();
    assert!(
        (depot_lat - 54.81).abs() < 0.01,
        "expected depot lat≈54.81, got {depot_lat}"
    );
}

#[tokio::test]
async fn create_and_list_start_point() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let resp = app
        .post_json(
            "/api/v1/routing/start-points",
            &serde_json::json!({"name": "Depot Süd", "lat": 54.75, "lon": 9.43, "watering_point": true}),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    let created: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(created["name"], "Depot Süd");
    assert_eq!(created["is_default"], false);
    assert_eq!(created["watering_point"], true);

    let list = app.get("/api/v1/routing/start-points").await;
    let arr: serde_json::Value = list.json().await.unwrap();
    let names: Vec<&str> = arr
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Depot Süd"));
}

#[tokio::test]
async fn set_default_moves_default_flag() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let list: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let arr = list.as_array().unwrap();
    // Find a non-default point ("Depot Nord").
    let target = arr.iter().find(|p| p["name"] == "Depot Nord").unwrap();
    let target_id = target["id"].as_str().unwrap();

    let resp = app
        .post_json(
            &format!("/api/v1/routing/start-points/{target_id}/default"),
            &serde_json::json!({}),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 204);

    let after: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let defaults: Vec<&str> = after
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["is_default"] == true)
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        defaults,
        vec!["Depot Nord"],
        "exactly one default, now Depot Nord"
    );
}

#[tokio::test]
async fn set_default_back_to_lower_ctid_row_succeeds() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let list: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let arr = list.as_array().unwrap();
    let find = |name: &str| {
        arr.iter().find(|p| p["name"] == name).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    // Betriebshof is the seeded default and was inserted first (lowest ctid).
    let betriebshof = find("Betriebshof Schleswiger Straße");
    let klaerwerk = find("Klärwerk Kielseng");

    // Move default forward to Klärwerk (higher ctid) — always worked.
    let r1 = app
        .post_json(
            &format!("/api/v1/routing/start-points/{klaerwerk}/default"),
            &serde_json::json!({}),
        )
        .await;
    assert_eq!(r1.status().as_u16(), 204);

    // Move default BACK to Betriebshof (lower ctid). The old single-statement
    // UPDATE would raise a transient unique violation (HTTP 409) here.
    let r2 = app
        .post_json(
            &format!("/api/v1/routing/start-points/{betriebshof}/default"),
            &serde_json::json!({}),
        )
        .await;
    assert_eq!(
        r2.status().as_u16(),
        204,
        "set_default must be order-independent"
    );

    let after: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let defaults: Vec<&str> = after
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["is_default"] == true)
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(defaults, vec!["Betriebshof Schleswiger Straße"]);
}

#[tokio::test]
async fn delete_default_start_point_is_rejected() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let list: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let default = list
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["is_default"] == true)
        .unwrap();
    let id = default["id"].as_str().unwrap();

    let resp = app
        .delete(&format!("/api/v1/routing/start-points/{id}"))
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn delete_non_default_start_point_succeeds() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let list: serde_json::Value = app
        .get("/api/v1/routing/start-points")
        .await
        .json()
        .await
        .unwrap();
    let target = list
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "Depot Nord")
        .unwrap();
    let id = target["id"].as_str().unwrap();

    let resp = app
        .delete(&format!("/api/v1/routing/start-points/{id}"))
        .await;
    assert_eq!(resp.status().as_u16(), 204);
}

#[tokio::test]
async fn set_default_unknown_id_returns_404() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;

    let resp = app
        .post_json(
            "/api/v1/routing/start-points/0190a8e9-7c4f-7000-8000-000000000000/default",
            &serde_json::json!({}),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn refill_stations_sent_to_streamlet_are_only_watering_points() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, &cid))
        .await;
    assert_eq!(resp.status().as_u16(), 201);

    let requests = streamlet.received_requests().await.unwrap();
    let last = requests.last().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&last.body).unwrap();
    let refills = body["problem"]["refill_stations"].as_array().unwrap();
    assert_eq!(
        refills.len(),
        2,
        "only the two watering_point depots are refill stations"
    );
    assert!(
        refills
            .iter()
            .all(|r| (r["location"]["lat"].as_f64().unwrap() - 54.81).abs() > 0.001),
        "Depot Nord (watering_point=false) must not be a refill station"
    );
}

#[tokio::test]
async fn preview_with_unknown_start_point_name_falls_back_to_default() {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cid = create_cluster_with_tree(&app).await;

    let resp = app
        .post_json(
            "/api/v1/watering-plans/route/preview",
            &serde_json::json!({
                "cluster_ids": [cid],
                "transporter_id": tid,
                "start_point_name": "Kein solcher Punkt"
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 200);

    let requests = streamlet.received_requests().await.unwrap();
    let last = requests.last().unwrap();
    let body: serde_json::Value = serde_json::from_slice(&last.body).unwrap();
    let depot_lat = body["problem"]["depots"][0]["location"]["lat"]
        .as_f64()
        .unwrap();
    assert!(
        (depot_lat - 54.76879).abs() < 0.001,
        "expected fallback to default depot Betriebshof (lat≈54.76879), got {depot_lat}"
    );
}
