use serde_json::{Value, json};
use server::service::sensor_parsers::SensorPayloadParser;
use uuid::Uuid;
use waste_server::sensor_parser::{DISTANCE_MODEL, DistanceSensorParser};

use crate::{
    helpers::TestApp,
    waste_bins::{bin_body, bin_target, create_bin, waste_app},
};

async fn create_distance_sensor(app: &TestApp, id: &str) -> String {
    let model_id: Uuid = sqlx::query_scalar("SELECT id FROM sensor_models WHERE name = $1")
        .bind(DISTANCE_MODEL)
        .fetch_one(&app.db_pool)
        .await
        .expect("the waste migration creates the model");
    let res = app
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
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    id.to_owned()
}

async fn activate_on_bin(app: &TestApp, sensor: &str, bin: &str) -> reqwest::Response {
    app.post_json(
        &format!("/api/v1/sensors/{sensor}/activate"),
        &bin_target(bin),
    )
    .await
}

async fn ingest_distance(app: &TestApp, sensor: &str, centimeters: f64) {
    let model_id: Uuid = sqlx::query_scalar("SELECT id FROM sensor_models WHERE name = $1")
        .bind(DISTANCE_MODEL)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(model_id))
        .await
        .expect("model loads");
    let ingest = DistanceSensorParser
        .parse(
            &model,
            &json!({ "distance_cm": centimeters, "battery": 3.6 }),
            domain::sensor::SensorId::new(sensor).unwrap(),
        )
        .expect("payload parses");
    app.state
        .sensor_service
        .ingest_reading(ingest)
        .await
        .expect("ingest succeeds");
}

async fn get_bin(app: &TestApp, id: &str) -> Value {
    app.get(&format!("/api/v1/waste-bins/{id}"))
        .await
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn the_distance_model_is_seeded_with_distance_and_battery() {
    let app = waste_app().await;
    let model_id: Uuid = sqlx::query_scalar("SELECT id FROM sensor_models WHERE name = $1")
        .bind(DISTANCE_MODEL)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    let body: Value = app
        .get(&format!("/api/v1/sensors/models/{model_id}"))
        .await
        .json()
        .await
        .unwrap();
    let mut abilities: Vec<_> = body["abilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["ability"].as_str().unwrap().to_owned(),
                a["depth_cm"].as_i64().unwrap(),
            )
        })
        .collect();
    abilities.sort();
    assert_eq!(
        abilities,
        [("battery".to_owned(), 0), ("distance".to_owned(), 0)]
    );
}

#[tokio::test]
async fn a_reading_sets_the_fill_level_of_the_attached_bin() {
    let app = waste_app().await;
    let mut body = bin_body("AB-1");
    body["depth_cm"] = json!(100);
    let res = app.post_json("/api/v1/waste-bins", &body).await;
    let bin: Value = res.json().await.unwrap();
    let bin = bin["id"].as_str().unwrap();
    let sensor = create_distance_sensor(&app, "eui-bin-1").await;
    assert_eq!(activate_on_bin(&app, &sensor, bin).await.status(), 200);

    ingest_distance(&app, &sensor, 20.0).await;

    let got = get_bin(&app, bin).await;
    assert_eq!(got["fill_percent"], 80);
    assert_eq!(got["fill_state"], "full");
    assert_eq!(got["sensor_id"], sensor);
}

#[tokio::test]
async fn a_bin_takes_only_one_sensor() {
    let app = waste_app().await;
    let bin = create_bin(&app, "AB-1").await;
    let bin = bin["id"].as_str().unwrap();
    let first = create_distance_sensor(&app, "eui-bin-a").await;
    let second = create_distance_sensor(&app, "eui-bin-b").await;
    assert_eq!(activate_on_bin(&app, &first, bin).await.status(), 200);

    let res = activate_on_bin(&app, &second, bin).await;
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "conflict.target_already_has_sensor"
    );
}

#[tokio::test]
async fn a_bin_comment_goes_through_the_core_target_route() {
    let app = waste_app().await;
    let bin = create_bin(&app, "AB-1").await;
    let bin = bin["id"].as_str().unwrap();
    let res = app
        .post_json(
            &format!("/api/v1/targets/waste/bin/{bin}/comments"),
            &json!({ "body": "Deckel klemmt" }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
}
