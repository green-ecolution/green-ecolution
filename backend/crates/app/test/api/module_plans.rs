use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app_with_modules, spawn_app_with_modules_and_routing},
    module_host::switch,
    settings_repo::child_org,
    test_module::{KEY, TestModule},
};

const ROOT: &str = "01980000-0000-7000-8000-000000000001";

async fn app() -> TestApp {
    let app = spawn_app_with_modules(vec![Box::new(TestModule::new(Default::default()))]).await;
    switch(&app, KEY, ROOT.parse().unwrap(), Some(true)).await;
    app
}

async fn thing_in(app: &TestApp, org: Uuid) -> Uuid {
    let res = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": org, "latitude": 54.78, "longitude": 9.43 }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    res.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn thing(app: &TestApp) -> Uuid {
    thing_in(app, ROOT.parse().unwrap()).await
}

async fn cargo_vehicle(app: &TestApp) -> Uuid {
    crate::vehicle::create_vehicle_with(app, json!({ "cargo_volume": 6000.0 })).await
}

async fn plan_in(app: &TestApp, stops: &[Uuid], vehicle: Uuid, org: Uuid) -> Value {
    let res = app
        .post_json(
            "/api/v1/plans",
            &json!({
                "module": "test_module",
                "date": "2026-11-02T08:00:00Z",
                "description": "Leerung",
                "transporter_id": vehicle,
                "stops": stops,
                "user_ids": [],
                "transporter_capacity_used": true,
                "organization_id": org
            }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    res.json().await.unwrap()
}

async fn plan(app: &TestApp, stops: &[Uuid], vehicle: Uuid) -> Value {
    plan_in(app, stops, vehicle, ROOT.parse().unwrap()).await
}

async fn put_status(app: &TestApp, id: &str, body: Value) -> reqwest::Response {
    app.put_json(&format!("/api/v1/plans/{id}"), &body).await
}

fn update_body(plan: &Value, status: &str, outcomes: Option<Value>) -> Value {
    json!({
        "date": plan["date"],
        "description": plan["description"],
        "transporter_id": plan["transporter"]["id"],
        "stops": plan["stops"],
        "user_ids": [],
        "transporter_capacity_used": true,
        "status": status,
        "cancellation_note": "",
        "outcomes": outcomes
    })
}

#[tokio::test]
async fn a_module_plan_runs_through_its_lifecycle() {
    let app = app().await;
    let (a, b) = (thing(&app).await, thing(&app).await);
    let vehicle = cargo_vehicle(&app).await;
    let plan = plan(&app, &[a, b], vehicle).await;
    let id = plan["id"].as_str().unwrap();
    assert_eq!(plan["module"], "test_module");

    assert_eq!(
        put_status(&app, id, update_body(&plan, "active", None))
            .await
            .status(),
        200
    );
    let outcomes = json!([
        { "stop": a, "completed": true, "amount": 120.0 },
        { "stop": b, "completed": false, "amount": null }
    ]);
    assert_eq!(
        put_status(&app, id, update_body(&plan, "finished", Some(outcomes)))
            .await
            .status(),
        200
    );

    let done: Value = app
        .get(&format!("/api/v1/plans/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(done["status"], "finished");
    let outcomes = done["outcomes"].as_array().unwrap();
    assert!(
        outcomes
            .iter()
            .any(|o| o["stop"] == json!(a) && o["completed"] == true && o["amount"] == 120.0)
    );
    assert!(
        outcomes
            .iter()
            .any(|o| o["stop"] == json!(b) && o["completed"] == false)
    );
}

#[tokio::test]
async fn finishing_without_every_outcome_is_refused() {
    let app = app().await;
    let (a, b) = (thing(&app).await, thing(&app).await);
    let plan = plan(&app, &[a, b], cargo_vehicle(&app).await).await;
    let id = plan["id"].as_str().unwrap();
    put_status(&app, id, update_body(&plan, "active", None)).await;
    let res = put_status(
        &app,
        id,
        update_body(
            &plan,
            "finished",
            Some(json!([{ "stop": a, "completed": true, "amount": 1.0 }])),
        ),
    )
    .await;
    assert_eq!(res.status(), 400);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "plan.outcome_missing"
    );
}

#[tokio::test]
async fn the_same_status_again_changes_nothing() {
    let app = app().await;
    let plan = plan(&app, &[thing(&app).await], cargo_vehicle(&app).await).await;
    let id = plan["id"].as_str().unwrap();
    let res = put_status(&app, id, update_body(&plan, "planned", None)).await;
    assert_eq!(res.status(), 200);
    let again: Value = app
        .get(&format!("/api/v1/plans/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(again["status"], "planned");
}

#[tokio::test]
async fn a_disabled_module_refuses_new_plans() {
    let app = app().await;
    let stop = thing(&app).await;
    let vehicle = cargo_vehicle(&app).await;
    switch(&app, KEY, ROOT.parse().unwrap(), Some(false)).await;
    let res = app
        .post_json(
            "/api/v1/plans",
            &json!({
                "module": "test_module", "date": "2026-11-02T08:00:00Z", "description": "",
                "transporter_id": vehicle, "stops": [stop], "user_ids": [],
                "transporter_capacity_used": true, "organization_id": ROOT
            }),
        )
        .await;
    assert_eq!(res.status(), 403);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "module.disabled"
    );
}

#[tokio::test]
async fn listing_shows_only_plans_of_the_requested_module() {
    let app = app().await;
    plan(&app, &[thing(&app).await], cargo_vehicle(&app).await).await;
    let res: Value = app
        .get("/api/v1/plans?module=irrigation")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(res["pagination"]["total_records"], 0);
    let res: Value = app
        .get("/api/v1/plans?module=test_module")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(res["pagination"]["total_records"], 1);
}

#[tokio::test]
async fn listing_leaves_out_plans_where_the_module_is_switched_off() {
    let app = app().await;
    let root: Uuid = ROOT.parse().unwrap();
    let child = child_org(&app, "Abfall Nord", root).await;
    let vehicle = cargo_vehicle(&app).await;
    let root_plan = plan(&app, &[thing(&app).await], vehicle).await;
    plan_in(&app, &[thing_in(&app, child).await], vehicle, child).await;

    switch(&app, KEY, child, Some(false)).await;

    let res: Value = app
        .get("/api/v1/plans?module=test_module")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(res["pagination"]["total_records"], 1);
    assert_eq!(res["data"][0]["id"], root_plan["id"]);
}

#[tokio::test]
async fn listing_needs_a_module() {
    let app = app().await;
    assert_eq!(app.get("/api/v1/plans").await.status(), 400);
    let res = app.get("/api/v1/plans?module=ghost").await;
    assert_eq!(res.status(), 422);
    assert_eq!(res.json::<Value>().await.unwrap()["code"], "module.unknown");
}

#[tokio::test]
async fn deleting_a_stop_object_removes_it_from_finished_plans_too() {
    let app = app().await;
    let (a, b) = (thing(&app).await, thing(&app).await);
    let plan = plan(&app, &[a, b], cargo_vehicle(&app).await).await;
    let id = plan["id"].as_str().unwrap();
    put_status(&app, id, update_body(&plan, "active", None)).await;
    let outcomes = json!([
        { "stop": a, "completed": true, "amount": 1.0 },
        { "stop": b, "completed": true, "amount": 2.0 }
    ]);
    put_status(&app, id, update_body(&plan, "finished", Some(outcomes))).await;

    assert_eq!(
        app.delete(&format!("/api/v1/test-module/things/{a}"))
            .await
            .status(),
        204
    );

    let done: Value = app
        .get(&format!("/api/v1/plans/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(done["stops"], json!([b]));
    assert_eq!(done["outcomes"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_vehicle_without_cargo_volume_does_not_fit_a_collection_plan() {
    let app = app().await;
    let stop = thing(&app).await;
    let water_only = crate::vehicle::create_vehicle_with(&app, json!({ "water": 3000.0 })).await;
    let res = app
        .post_json(
            "/api/v1/plans",
            &json!({
                "module": "test_module", "date": "2026-11-02T08:00:00Z", "description": "",
                "transporter_id": water_only, "stops": [stop], "user_ids": [],
                "transporter_capacity_used": true, "organization_id": ROOT
            }),
        )
        .await;
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "vehicle.capacity_missing"
    );
}

#[tokio::test]
async fn a_collection_route_without_an_unloading_station_is_refused() {
    let streamlet = wiremock::MockServer::start().await;
    let app = spawn_app_with_modules_and_routing(
        vec![Box::new(TestModule::new(Default::default()))],
        &streamlet.uri(),
    )
    .await;
    switch(&app, KEY, ROOT.parse().unwrap(), Some(true)).await;
    let stop = thing(&app).await;
    let vehicle = cargo_vehicle(&app).await;

    let res = app
        .post_json(
            "/api/v1/plans/route/preview",
            &json!({ "module": "test_module", "stops": [stop], "transporter_id": vehicle }),
        )
        .await;

    assert_eq!(res.status(), 422);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "routing.no_station_for_capacity");
    assert!(streamlet.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_collection_route_with_an_unloading_station_asks_the_optimizer_to_return_full() {
    let streamlet = wiremock::MockServer::start().await;
    let app = spawn_app_with_modules_and_routing(
        vec![Box::new(TestModule::new(Default::default()))],
        &streamlet.uri(),
    )
    .await;
    switch(&app, KEY, ROOT.parse().unwrap(), Some(true)).await;
    let station = app
        .post_json(
            "/api/v1/routing/start-points",
            &json!({ "name": "Deponie", "lat": 54.8, "lon": 9.4, "replenishes": ["cargo_volume"] }),
        )
        .await;
    assert_eq!(station.status(), 201, "{}", station.text().await.unwrap());
    let stop = thing(&app).await;
    let vehicle = cargo_vehicle(&app).await;

    app.post_json(
        "/api/v1/plans/route/preview",
        &json!({ "module": "test_module", "stops": [stop], "transporter_id": vehicle }),
    )
    .await;

    let requests = streamlet.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["problem"]["vehicles"][0]["return_full"], true);
}
