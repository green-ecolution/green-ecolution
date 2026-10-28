use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::{
    helpers::{TestApp, spawn_app_with_modules_and_routing},
    module_host::switch,
    module_plans::{put_status, update_body},
    organizations::ROOT_ORG_ID,
    routing::streamlet_ok,
    waste_bins::{create_bin, id_of, set_fill, waste_app},
};

const KEY: &str = "waste";

async fn collection_plan(app: &TestApp, stops: &[Uuid], vehicle: Uuid) -> Value {
    let res = app
        .post_json(
            "/api/v1/plans",
            &json!({
                "module": KEY,
                "date": "2026-11-02T08:00:00Z",
                "description": "Leerung",
                "transporter_id": vehicle,
                "stops": stops,
                "user_ids": [],
                "transporter_capacity_used": true,
                "organization_id": ROOT_ORG_ID
            }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    res.json().await.unwrap()
}

async fn routing_waste_app() -> (TestApp, MockServer) {
    let streamlet = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(ResponseTemplate::new(200).set_body_json(streamlet_ok()))
        .mount(&streamlet)
        .await;
    let app = spawn_app_with_modules_and_routing(vec![], &streamlet.uri()).await;
    assert_eq!(
        switch(&app, KEY, ROOT_ORG_ID.parse().unwrap(), Some(true))
            .await
            .status(),
        200
    );
    let station = app
        .post_json(
            "/api/v1/routing/start-points",
            &json!({ "name": "Wertstoffhof", "lat": 54.8, "lon": 9.4, "replenishes": ["cargo_volume"] }),
        )
        .await;
    assert_eq!(station.status(), 201, "{}", station.text().await.unwrap());
    (app, streamlet)
}

#[tokio::test]
async fn a_collection_tour_asks_streamlet_to_end_at_an_unloading_station() {
    let (app, streamlet) = routing_waste_app().await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let vehicle =
        crate::vehicle::create_vehicle_with(&app, json!({ "cargo_volume": 6000.0 })).await;

    let res = app
        .post_json(
            "/api/v1/plans/route/preview",
            &json!({ "module": KEY, "stops": [bin], "transporter_id": vehicle }),
        )
        .await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());

    let requests = streamlet.received_requests().await.unwrap();
    let request: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(request["problem"]["vehicles"][0]["return_full"], true);
    let refill_stations = request["problem"]["refill_stations"].as_array().unwrap();
    assert_eq!(refill_stations.len(), 1);
}

#[tokio::test]
async fn a_plan_carries_the_filled_volume_as_its_demand() {
    let (app, _streamlet) = routing_waste_app().await;
    let half = id_of(&create_bin(&app, "AB-1").await);
    let unknown = id_of(&create_bin(&app, "AB-2").await);
    set_fill(&app, half, Some(0.5), Some(0), None).await;
    let vehicle =
        crate::vehicle::create_vehicle_with(&app, json!({ "cargo_volume": 6000.0 })).await;

    let plan = collection_plan(&app, &[half, unknown], vehicle).await;

    // 1100 l bins: half full plus an unmeasured one counted as full.
    assert_eq!(plan["total_demand"], 550.0 + 1100.0, "{plan}");
}

#[tokio::test]
async fn finishing_a_tour_empties_only_completed_bins() {
    let app = waste_app().await;
    let (a, b) = (
        id_of(&create_bin(&app, "AB-1").await),
        id_of(&create_bin(&app, "AB-2").await),
    );
    set_fill(&app, a, Some(0.9), Some(0), None).await;
    set_fill(&app, b, Some(0.9), Some(0), None).await;
    let vehicle =
        crate::vehicle::create_vehicle_with(&app, json!({ "cargo_volume": 6000.0 })).await;
    let plan = collection_plan(&app, &[a, b], vehicle).await;
    let id = plan["id"].as_str().unwrap();

    let res = put_status(&app, id, update_body(&plan, "active", None)).await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());
    let outcomes = json!([
        { "stop": a, "completed": true, "amount": 900.0 },
        { "stop": b, "completed": false, "amount": null }
    ]);
    let res = put_status(&app, id, update_body(&plan, "finished", Some(outcomes))).await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());

    let emptied: Value = app
        .get(&format!("/api/v1/waste-bins/{a}"))
        .await
        .json()
        .await
        .unwrap();
    assert!(emptied["last_emptied_at"].is_string(), "{emptied}");
    assert_eq!(emptied["fill_state"], "empty");

    let untouched: Value = app
        .get(&format!("/api/v1/waste-bins/{b}"))
        .await
        .json()
        .await
        .unwrap();
    assert!(untouched["last_emptied_at"].is_null(), "{untouched}");
    assert_eq!(untouched["fill_state"], "full");
}

#[tokio::test]
async fn statistics_count_bins() {
    let app = waste_app().await;
    create_bin(&app, "AB-1").await;

    let body: Value = app
        .get("/api/v1/info/statistics")
        .await
        .json()
        .await
        .unwrap();

    let entry = body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["module"] == KEY && m["kind"] == "bin")
        .unwrap_or_else(|| panic!("no waste entry in {body}"));
    assert_eq!(entry["count"], 1);
}
