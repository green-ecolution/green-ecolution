use std::sync::{Arc, atomic::Ordering};
use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use crate::{
    helpers::{spawn_app_with_modules, try_spawn_app_with_modules},
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
    test_module::{CollidingModule, KEY, LEVEL_MODEL, ParserOnlyModule, Probe, TestModule},
};

async fn app_with_test_module() -> (crate::helpers::TestApp, Arc<Probe>) {
    let probe = Arc::new(Probe::default());
    let app = spawn_app_with_modules(vec![Box::new(TestModule::new(probe.clone()))]).await;
    (app, probe)
}

pub async fn switch(
    app: &crate::helpers::TestApp,
    org: Uuid,
    enabled: Option<bool>,
) -> reqwest::Response {
    app.put_json(
        &format!("/api/v1/organizations/{org}/modules"),
        &json!({ "modules": { KEY: enabled } }),
    )
    .await
}

#[tokio::test]
async fn a_registered_module_is_listed_and_off_by_default() {
    let (app, _) = app_with_test_module().await;
    let body: serde_json::Value = app.get("/api/v1/modules").await.json().await.unwrap();
    assert_eq!(body["modules"], json!(["irrigation", KEY]));

    let org: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{ROOT_ORG_ID}/modules"))
        .await
        .json()
        .await
        .unwrap();
    let entry = org["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == KEY)
        .unwrap();
    assert_eq!(entry["enabled"], false);
    assert_eq!(entry["origin"], "default");
}

#[tokio::test]
async fn writes_into_an_organization_without_the_module_are_refused() {
    let (app, _) = app_with_test_module().await;
    let org = child_org(&app, "Ohne", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let resp = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "module.disabled");

    assert_eq!(switch(&app, org, Some(true)).await.status(), 200);
    let resp = app
        .post_json(
            "/api/v1/test-module/things",
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn activation_at_the_root_reaches_three_levels_down() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Mitte", root).await;
    let leaf = child_org(&app, "Blatt", mid).await;

    assert_eq!(switch(&app, root, Some(true)).await.status(), 200);

    let body: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{leaf}/modules"))
        .await
        .json()
        .await
        .unwrap();
    let entry = body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == KEY)
        .unwrap();
    assert_eq!(entry["enabled"], true);
    assert_eq!(entry["origin"], "inherited");
    assert_eq!(entry["source"]["id"], ROOT_ORG_ID);
}

#[tokio::test]
async fn a_locked_subtree_cannot_switch_modules() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Sperre", root).await;
    let leaf = child_org(&app, "Darunter", mid).await;
    let lock = app
        .put_json(
            &format!("/api/v1/organizations/{mid}/settings"),
            &json!({ "descendants_may_override": false }),
        )
        .await;
    assert_eq!(lock.status(), 200);

    let resp = switch(&app, leaf, Some(true)).await;
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn reads_only_show_organizations_with_the_module_active() {
    let (app, _) = app_with_test_module().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let on = child_org(&app, "An", root).await;
    let off = child_org(&app, "Aus", root).await;
    assert_eq!(switch(&app, on, Some(true)).await.status(), 200);
    assert_eq!(switch(&app, off, Some(true)).await.status(), 200);
    for org in [on, off] {
        let created = app
            .post_json(
                "/api/v1/test-module/things",
                &json!({ "organization_id": org }),
            )
            .await;
        assert_eq!(created.status(), 201);
    }
    assert_eq!(switch(&app, off, Some(false)).await.status(), 200);

    let things: Vec<serde_json::Value> = app
        .get("/api/v1/test-module/things")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(things.len(), 1);
    assert_eq!(things[0]["organization_id"], on.to_string());
}

#[tokio::test]
async fn module_handlers_receive_core_events_and_tasks_run() {
    let (app, probe) = app_with_test_module().await;
    child_org(&app, "Neu", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;
    assert_eq!(probe.organizations_created.load(Ordering::SeqCst), 1);

    for _ in 0..50 {
        if probe.task_ran.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("background task did not run");
}

#[tokio::test]
async fn a_module_shadowing_a_core_route_stops_the_start() {
    let err = match try_spawn_app_with_modules(vec![Box::new(CollidingModule)]).await {
        Ok(_) => panic!("startup must refuse a colliding module"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("/organizations"), "{err}");
}

#[tokio::test]
async fn a_second_parser_for_the_same_model_stops_the_start() {
    let err = match try_spawn_app_with_modules(vec![
        Box::new(TestModule::new(Default::default())),
        Box::new(ParserOnlyModule::new("parser_twin", LEVEL_MODEL)),
    ])
    .await
    {
        Ok(_) => panic!("startup must refuse two parsers for one model"),
        Err(err) => err,
    };
    assert!(err.to_string().contains(LEVEL_MODEL), "{err}");
}

async fn level_sensor(
    app: &crate::helpers::TestApp,
    id: &str,
) -> domain::sensor_model::SensorModel {
    let model_id: Uuid = sqlx::query_scalar("SELECT id FROM sensor_models WHERE name = $1")
        .bind(LEVEL_MODEL)
        .fetch_one(&app.db_pool)
        .await
        .expect("the test module migration creates the model");
    let resp = app
        .post_json(
            "/api/v1/sensors",
            &json!({
                "id": id,
                "sensor_type": "lorawan",
                "model_id": model_id,
                "lorawan": {
                    "serial_number": "SN",
                    "dev_eui": "a81758fffe0c3b52",
                    "app_eui": "70b3d57ed00abcd1",
                    "app_key": "00112233445566778899aabbccddeeff"
                }
            }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 201);
    app.state
        .sensor_service
        .model_by_id(domain::Id::new(model_id))
        .await
        .expect("model loads")
}

async fn ingest_distance(
    app: &crate::helpers::TestApp,
    model: &domain::sensor_model::SensorModel,
    sensor_id: &str,
    centimeters: f64,
) {
    use domain::sensor::data::DistanceReading;
    use domain::sensor_model::SensorAbilityName;

    let ability = model
        .ability_id_for(SensorAbilityName::Distance, 0)
        .expect("Test-Level measures distance at 0 cm");
    app.state
        .sensor_service
        .ingest_reading(server::service::sensor_service::ReadingIngest {
            sensor_id: domain::sensor::SensorId::new(sensor_id).unwrap(),
            raw_payload: json!({ "distance_cm": centimeters }),
            normalized: vec![domain::sensor::repository::NormalizedValue {
                model_ability_id: ability,
                value: rust_decimal::Decimal::from_f64_retain(centimeters).unwrap(),
                issue: None,
            }],
            typed: domain::events::SensorReadings::Distances(vec![DistanceReading {
                depth_cm: 0,
                centimeters,
            }]),
        })
        .await
        .expect("ingest succeeds");
}

async fn wait_for_events(probe: &Probe, expected: usize) {
    for _ in 0..50 {
        if probe.sensor_data_received.load(Ordering::SeqCst) >= expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn a_distance_above_the_physical_range_is_flagged_and_publishes_no_event() {
    let (app, probe) = app_with_test_module().await;
    let model = level_sensor(&app, "eui-level-high").await;

    ingest_distance(&app, &model, "eui-level-high", 1200.0).await;
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(probe.sensor_data_received.load(Ordering::SeqCst), 0);
    let plausible: bool = sqlx::query_scalar(
        "SELECT dav.plausible FROM sensor_data_ability_values dav
         JOIN sensor_data sd ON sd.id = dav.sensor_data_id
         WHERE sd.sensor_id = 'eui-level-high'",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert!(!plausible);
}

#[tokio::test]
async fn a_plausible_distance_publishes_the_event() {
    let (app, probe) = app_with_test_module().await;
    let model = level_sensor(&app, "eui-level-ok").await;

    ingest_distance(&app, &model, "eui-level-ok", 50.0).await;
    wait_for_events(&probe, 1).await;

    assert_eq!(probe.sensor_data_received.load(Ordering::SeqCst), 1);
}
