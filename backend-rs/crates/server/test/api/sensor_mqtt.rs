use crate::helpers::{TestApp, spawn_app};
use domain::{
    events::SensorReadings,
    sensor::{SensorId, data::VolumetricReading, repository::NormalizedValue},
};
use rust_decimal::Decimal;
use serde_json::json;
use server::service::sensor_service::ReadingIngest;

fn create_body(id: &str, model_id: i32) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "SN",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

async fn create_sensor(app: &TestApp, id: &str, model_id: i32) {
    let r = app
        .post_json("/api/v1/sensors", &create_body(id, model_id))
        .await;
    assert_eq!(r.status().as_u16(), 201, "sensor create failed");
}

async fn ability_value_count(app: &TestApp, sensor_id: &str) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM sensor_data_ability_values dav
           JOIN sensor_data sd ON sd.id = dav.sensor_data_id
           WHERE sd.sensor_id = $1"#,
        sensor_id
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

async fn insert_tree(app: &TestApp, number: &str) -> i32 {
    // planted this year so the year=0 calibration table applies and the
    // watering handler produces a status (not BeyondMonitoring).
    let planting_year: i32 = chrono::Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap();
    sqlx::query_scalar!(
        r#"INSERT INTO trees (planting_year, species, number, latitude, longitude, geometry, description)
        VALUES ($1, 'Eiche', $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), 'Test')
        RETURNING id"#,
        planting_year,
        number,
        54.79_f64,
        9.45_f64,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn ecodrizzler_handle_message_writes_normalized_values_and_bumps_online() {
    let app = spawn_app().await;
    create_sensor(&app, "eui-mqtt-eco", 1).await;

    let payload = json!({
        "device": "eui-mqtt-eco",
        "battery": 3.6,
        "humidity": 0.4,
        "temperature": 18.0,
        "latitude": 54.79,
        "longitude": 9.45,
        "watermarks": [
            {"depth": 30, "resistance": 1200, "centibar": 45},
            {"depth": 60, "resistance": 800,  "centibar": 28},
            {"depth": 90, "resistance": 600,  "centibar": 18},
        ]
    });
    app.handle_mqtt_message(payload)
        .await
        .expect("ingest succeeds");

    // 3 watermarks + 1 temperature + 1 humidity = 5 normalized values
    let count = ability_value_count(&app, "eui-mqtt-eco").await;
    assert_eq!(count, 5, "expected 5 normalized values, got {count}");

    // Sensor was prepared; handle_message bumps to online for any ingest.
    let view = app.get("/api/v1/sensors/eui-mqtt-eco").await;
    let body: serde_json::Value = view.json().await.unwrap();
    assert_eq!(body["status"], "online");
}

#[tokio::test]
async fn ecodrizzler_handle_message_updates_linked_tree_watering_status() {
    let app = spawn_app().await;
    create_sensor(&app, "eui-mqtt-eco-2", 1).await;
    let tree_id = insert_tree(&app, "T-MQ-ECO-2").await;

    let r = app
        .post_json(
            "/api/v1/sensors/eui-mqtt-eco-2/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r.status().as_u16(), 200);

    // 5 centibar across all depths → score=0 per depth (year 0/1 calibration) → Good.
    let payload = json!({
        "device": "eui-mqtt-eco-2",
        "battery": 3.6,
        "humidity": 0.4,
        "temperature": 18.0,
        "latitude": 54.79,
        "longitude": 9.45,
        "watermarks": [
            {"depth": 30, "resistance": 0, "centibar": 5},
            {"depth": 60, "resistance": 0, "centibar": 5},
            {"depth": 90, "resistance": 0, "centibar": 5},
        ]
    });
    app.handle_mqtt_message(payload).await.unwrap();

    let status: String = sqlx::query_scalar!(
        r#"SELECT watering_status::text AS "ws!" FROM trees WHERE number = 'T-MQ-ECO-2'"#
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(status, "good");
}

#[tokio::test]
async fn ges_1000_ingest_writes_volumetric_normalized_values() {
    let app = spawn_app().await;
    create_sensor(&app, "eui-mqtt-ges", 2).await;
    // `handle_message` is EcoDrizzler-specific; the GES-1000 dispatch lives in
    // `infra::mqtt::build_ges_1000` which is private. Call `ingest_reading`
    // directly with the pre-built normalized values — that is what the MQTT
    // adapter would do after parsing.
    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(2))
        .await
        .unwrap();
    let ab_30 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 30)
        .unwrap();
    let ab_90 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 90)
        .unwrap();

    let normalized = vec![
        NormalizedValue {
            model_ability_id: ab_30,
            value: Decimal::from_f64_retain(42.0).unwrap(),
        },
        NormalizedValue {
            model_ability_id: ab_90,
            value: Decimal::from_f64_retain(25.0).unwrap(),
        },
    ];
    let typed = SensorReadings::Volumetrics(vec![
        VolumetricReading {
            depth_cm: 30,
            moisture_percent: 42.0,
        },
        VolumetricReading {
            depth_cm: 90,
            moisture_percent: 25.0,
        },
    ]);
    let raw_payload = json!({
        "device": "eui-mqtt-ges",
        "readings": [
            {"ability": "soil_moisture", "depth": 30, "value": 42.0},
            {"ability": "soil_moisture", "depth": 90, "value": 25.0},
        ]
    });

    app.state
        .sensor_service
        .ingest_reading(ReadingIngest {
            sensor_id: SensorId::new("eui-mqtt-ges").unwrap(),
            raw_payload,
            normalized,
            typed,
        })
        .await
        .unwrap();

    let count = ability_value_count(&app, "eui-mqtt-ges").await;
    assert_eq!(count, 2);
}

#[tokio::test]
async fn prepared_sensor_ingest_persists_reading_without_tree_link() {
    let app = spawn_app().await;
    create_sensor(&app, "eui-mqtt-prep", 2).await;

    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(2))
        .await
        .unwrap();
    let ab_30 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 30)
        .unwrap();
    let normalized = vec![NormalizedValue {
        model_ability_id: ab_30,
        value: Decimal::from_f64_retain(33.0).unwrap(),
    }];

    app.state
        .sensor_service
        .ingest_reading(ReadingIngest {
            sensor_id: SensorId::new("eui-mqtt-prep").unwrap(),
            raw_payload: json!({"device": "eui-mqtt-prep"}),
            normalized,
            typed: SensorReadings::Volumetrics(vec![VolumetricReading {
                depth_cm: 30,
                moisture_percent: 33.0,
            }]),
        })
        .await
        .unwrap();

    // sensor_data row exists.
    let reading_count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM sensor_data WHERE sensor_id = $1"#,
        "eui-mqtt-prep"
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(reading_count, 1);

    // Direct ingest_reading does not bump_online; sensor stays prepared.
    let view = app.get("/api/v1/sensors/eui-mqtt-prep").await;
    let body: serde_json::Value = view.json().await.unwrap();
    assert_eq!(body["status"], "prepared");
}
