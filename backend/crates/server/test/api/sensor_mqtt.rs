use crate::helpers::{TestApp, spawn_app};
use domain::{
    Id,
    cluster::{ClusterAddress, SoilCondition, TreeClusterUpdate},
    events::SensorReadings,
    sensor::{SensorId, data::VolumetricReading, repository::NormalizedValue},
    shared::provenance::Provenance,
};
use rust_decimal::Decimal;
use serde_json::json;
use server::service::sensor_service::ReadingIngest;
use uuid::Uuid;

fn create_body(id: &str, model_id: Uuid) -> serde_json::Value {
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

async fn create_sensor(app: &TestApp, id: &str, model_id: Uuid) {
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

async fn insert_tree(app: &TestApp, number: &str) -> Uuid {
    // planted this year so the year=0 calibration table applies and the
    // watering handler produces a status (not BeyondMonitoring).
    let planting_year: i32 = chrono::Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap();
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES ($1, $2, 'Eiche', $3, $4, $5, ST_SetSRID(ST_MakePoint($5, $4), 4326), 'Test')"#,
        id,
        planting_year,
        number,
        54.79_f64,
        9.45_f64,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn ecodrizzler_ingest_writes_normalized_values_and_derives_online() {
    let app = spawn_app().await;
    let model_id = app.ecodrizzler_model_id().await;
    create_sensor(&app, "eui-mqtt-eco", model_id).await;
    let tree_id = insert_tree(&app, "T-MQ-ECO-1").await;
    let r = app
        .post_json(
            "/api/v1/sensors/eui-mqtt-eco/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r.status().as_u16(), 200);

    app.ingest_ecodrizzler("eui-mqtt-eco", 45)
        .await
        .expect("ingest succeeds");

    // 3 watermarks + 1 temperature + 1 humidity = 5 normalized values
    let count = ability_value_count(&app, "eui-mqtt-eco").await;
    assert_eq!(count, 5, "expected 5 normalized values, got {count}");

    let view = app.get("/api/v1/sensors/eui-mqtt-eco").await;
    let body: serde_json::Value = view.json().await.unwrap();
    assert_eq!(body["status"], "online");
}

#[tokio::test]
async fn ecodrizzler_ingest_updates_linked_tree_watering_status() {
    let app = spawn_app().await;
    let model_id = app.ecodrizzler_model_id().await;
    create_sensor(&app, "eui-mqtt-eco-2", model_id).await;
    let tree_id = insert_tree(&app, "T-MQ-ECO-2").await;

    let r = app
        .post_json(
            "/api/v1/sensors/eui-mqtt-eco-2/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(r.status().as_u16(), 200);

    // 5 centibar across all depths → score=0 per depth (year 0/1 calibration) → Good.
    app.ingest_ecodrizzler("eui-mqtt-eco-2", 5).await.unwrap();

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
    let model_id = app.ges_1000_model_id().await;
    create_sensor(&app, "eui-mqtt-ges", model_id).await;
    // The GES-1000 dispatch lives in `infra::mqtt::build_ges_1000`, which is
    // private; call `ingest_reading` with pre-built normalized values instead.
    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(model_id))
        .await
        .unwrap();
    let ab_40 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 40)
        .unwrap();
    let ab_80 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 80)
        .unwrap();

    let normalized = vec![
        NormalizedValue {
            model_ability_id: ab_40,
            value: Decimal::from_f64_retain(42.0).unwrap(),
        },
        NormalizedValue {
            model_ability_id: ab_80,
            value: Decimal::from_f64_retain(25.0).unwrap(),
        },
    ];
    let typed = SensorReadings::Volumetrics(vec![
        VolumetricReading {
            depth_cm: 40,
            moisture_percent: 42.0,
        },
        VolumetricReading {
            depth_cm: 80,
            moisture_percent: 25.0,
        },
    ]);
    let raw_payload = json!({
        "device": "eui-mqtt-ges",
        "readings": [
            {"ability": "soil_moisture", "depth": 40, "value": 42.0},
            {"ability": "soil_moisture", "depth": 80, "value": 25.0},
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
    let model_id = app.ges_1000_model_id().await;
    create_sensor(&app, "eui-mqtt-prep", model_id).await;

    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(model_id))
        .await
        .unwrap();
    let ab_40 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 40)
        .unwrap();
    let normalized = vec![NormalizedValue {
        model_ability_id: ab_40,
        value: Decimal::from_f64_retain(33.0).unwrap(),
    }];

    app.state
        .sensor_service
        .ingest_reading(ReadingIngest {
            sensor_id: SensorId::new("eui-mqtt-prep").unwrap(),
            raw_payload: json!({"device": "eui-mqtt-prep"}),
            normalized,
            typed: SensorReadings::Volumetrics(vec![VolumetricReading {
                depth_cm: 40,
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

    // Never activated → derived status stays prepared regardless of readings.
    let view = app.get("/api/v1/sensors/eui-mqtt-prep").await;
    let body: serde_json::Value = view.json().await.unwrap();
    assert_eq!(body["status"], "prepared");
}

/// Inserts a cluster with a given KA5 soil_condition; returns its id.
async fn insert_cluster_with_soil(app: &TestApp, soil: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO tree_clusters
             (id, name, address, description, moisture_level, soil_condition, watering_status)
           VALUES ($1, 'C', 'A', 'D', 0.5, $2::tree_soil_condition, 'unknown')"#,
    )
    .bind(id)
    .bind(soil)
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

/// Inserts a tree planted `years_ago` years ago, linked to `cluster_id`.
async fn insert_tree_in_cluster(
    app: &TestApp,
    number: &str,
    cluster_id: Uuid,
    years_ago: i32,
) -> Uuid {
    let planting_year: i32 = chrono::Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse::<i32>()
        .unwrap()
        - years_ago;
    let id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO trees
             (id, tree_cluster_id, planting_year, species, number, latitude, longitude, geometry, description)
           VALUES ($1, $2, $3, 'Eiche', $4, $5, $6, ST_SetSRID(ST_MakePoint($6, $5), 4326), 'T')"#,
    )
    .bind(id)
    .bind(cluster_id)
    .bind(planting_year)
    .bind(number)
    .bind(54.79_f64)
    .bind(9.45_f64)
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

async fn ingest_moisture(app: &TestApp, sensor: &str, model_id: Uuid, v40: f64, v80: f64) {
    let model = app
        .state
        .sensor_service
        .model_by_id(domain::Id::new(model_id))
        .await
        .unwrap();
    let ab_40 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 40)
        .unwrap();
    let ab_80 = model
        .ability_id_for(domain::sensor_model::SensorAbilityName::SoilMoisture, 80)
        .unwrap();
    app.state
        .sensor_service
        .ingest_reading(ReadingIngest {
            sensor_id: SensorId::new(sensor).unwrap(),
            raw_payload: json!({ "device": sensor }),
            normalized: vec![
                NormalizedValue {
                    model_ability_id: ab_40,
                    value: Decimal::from_f64_retain(v40).unwrap(),
                },
                NormalizedValue {
                    model_ability_id: ab_80,
                    value: Decimal::from_f64_retain(v80).unwrap(),
                },
            ],
            typed: SensorReadings::Volumetrics(vec![
                VolumetricReading {
                    depth_cm: 40,
                    moisture_percent: v40,
                },
                VolumetricReading {
                    depth_cm: 80,
                    moisture_percent: v80,
                },
            ]),
        })
        .await
        .unwrap();
}

async fn watering_status(app: &TestApp, number: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT watering_status::text FROM trees WHERE number = $1")
        .bind(number)
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn volumetric_status_depends_on_soil_and_age() {
    let app = spawn_app().await;
    let model_id = app.ges_1000_model_id().await;
    // Uu @ both depths: VWC_min=20, VWC_crit=18.
    let cluster = insert_cluster_with_soil(&app, "Uu").await;

    // Established tree (5y): worst-case over 40 cm (Good, 25) + 80 cm (Bad, 15) → bad.
    create_sensor(&app, "eui-soil-old", model_id).await;
    let old_tree = insert_tree_in_cluster(&app, "T-SOIL-OLD", cluster, 5).await;
    app.post_json(
        "/api/v1/sensors/eui-soil-old/activate",
        &json!({ "tree_id": old_tree }),
    )
    .await;
    ingest_moisture(&app, "eui-soil-old", model_id, 25.0, 15.0).await;
    assert_eq!(watering_status(&app, "T-SOIL-OLD").await, "bad");

    // Young tree (0y): only 40 cm (Good, 25) counts → good, despite the dry 80 cm probe.
    create_sensor(&app, "eui-soil-young", model_id).await;
    let young_tree = insert_tree_in_cluster(&app, "T-SOIL-YOUNG", cluster, 0).await;
    app.post_json(
        "/api/v1/sensors/eui-soil-young/activate",
        &json!({ "tree_id": young_tree }),
    )
    .await;
    ingest_moisture(&app, "eui-soil-young", model_id, 25.0, 15.0).await;
    assert_eq!(watering_status(&app, "T-SOIL-YOUNG").await, "good");
}

async fn cluster_watering_status(app: &TestApp, cluster_id: Uuid) -> String {
    sqlx::query_scalar::<_, String>("SELECT watering_status::text FROM tree_clusters WHERE id = $1")
        .bind(cluster_id)
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
}

/// Changing a cluster's soil condition must re-derive the watering status of
/// member trees and roll it up to the cluster.
///
/// Setup: Su3 cluster, established tree (5 y), moisture 13% at 40 + 80 cm.
/// Under Su3 (min=12): 13% → Good for both depths → tree Good, cluster Good.
/// After soil change to Uu (min=20): 13% → Bad for both depths → tree Bad, cluster Bad.
#[tokio::test]
async fn soil_condition_change_retriggers_tree_and_cluster_status() {
    let app = spawn_app().await;
    let model_id = app.ges_1000_model_id().await;

    // Su3: min @ 40 cm = 12, min @ 80 cm ≈ 10.8; 13 % is Good at both depths.
    let cluster_id = insert_cluster_with_soil(&app, "Su3").await;

    create_sensor(&app, "eui-soil-change", model_id).await;
    let tree_id = insert_tree_in_cluster(&app, "T-SOIL-CHG", cluster_id, 5).await;
    app.post_json(
        "/api/v1/sensors/eui-soil-change/activate",
        &json!({ "tree_id": tree_id }),
    )
    .await;

    // 13% moisture at both depths → Good under Su3.
    ingest_moisture(&app, "eui-soil-change", model_id, 13.0, 13.0).await;
    assert_eq!(
        watering_status(&app, "T-SOIL-CHG").await,
        "good",
        "pre-condition: tree should be good under Su3"
    );
    assert_eq!(
        cluster_watering_status(&app, cluster_id).await,
        "good",
        "pre-condition: cluster should be good under Su3"
    );

    // Change soil to Uu: min @ 40 cm = 20, min @ 80 cm = 20; 13% is Bad at both.
    let cluster = app
        .state
        .cluster_service
        .by_id(Id::new(cluster_id))
        .await
        .unwrap();
    app.state
        .cluster_service
        .replace(
            Id::new(cluster_id),
            TreeClusterUpdate {
                name: cluster.name.clone(),
                address: ClusterAddress::new(cluster.address.as_str()).unwrap(),
                description: cluster.description.clone(),
                soil_condition: Some(SoilCondition::Uu),
                tree_ids: cluster.tree_ids.clone(),
                provenance: Provenance::default(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        watering_status(&app, "T-SOIL-CHG").await,
        "bad",
        "tree must be bad after soil changed to Uu"
    );
    assert_eq!(
        cluster_watering_status(&app, cluster_id).await,
        "bad",
        "cluster must be bad after soil changed to Uu"
    );
}
