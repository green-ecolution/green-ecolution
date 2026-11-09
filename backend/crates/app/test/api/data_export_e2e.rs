use serde_json::{Value, json};
use uuid::Uuid;

use crate::{helpers::spawn_app_with_data_export, organizations::ROOT_ORG_ID};

/// Needs a running FROST (`just frost-up`); `just test-data-export` sets the URL.
#[tokio::test]
#[ignore]
async fn a_tree_reaches_the_local_frost_server_with_its_watering_status() {
    let frost = std::env::var("GE_TEST_FROST_URL").expect("GE_TEST_FROST_URL");
    let app = spawn_app_with_data_export(&["localhost:8180"]).await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tree = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO trees (id, number, species, planting_year, latitude, longitude, watering_status, organization_id)
         VALUES ($1, 'E2E', 'Linde', 2010, 54.79, 9.43, 'moderate', $2)",
    )
    .bind(tree)
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();
    let put = app
        .put_json(
            &format!("/api/v1/organizations/{root}/data-export"),
            &json!({ "enabled": true, "base_url": frost, "auth": { "kind": "none" } }),
        )
        .await;
    assert_eq!(put.status(), 200);

    let sensor = format!("eui-{}", Uuid::now_v7().simple());
    sqlx::query(
        "INSERT INTO sensors (id, model_id, type, organization_id)
         SELECT $1, id, 'lorawan', $2 FROM sensor_models WHERE name = 'GES-1000'",
    )
    .bind(&sensor)
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO sensor_attachments (sensor_id, module_key, target_kind, target_id) VALUES ($1, 'irrigation', 'tree', $2)")
        .bind(&sensor)
        .bind(tree)
        .execute(&app.db_pool)
        .await
        .unwrap();
    let reading = Uuid::now_v7();
    sqlx::query("INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, '{}')")
        .bind(reading)
        .bind(&sensor)
        .execute(&app.db_pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sensor_data_ability_values (sensor_data_id, sensor_model_ability_id, value, plausible, quality_reason)
         SELECT $1, ma.id, CASE WHEN ma.depth_cm = 40 THEN 42 ELSE 71 END, ma.depth_cm = 40, CASE WHEN ma.depth_cm = 40 THEN NULL ELSE 'out of range' END
         FROM sensor_model_abilities ma
         JOIN sensor_models m ON m.id = ma.sensor_model_id
         JOIN sensor_abilities a ON a.id = ma.sensor_ability_id
         WHERE m.name = 'GES-1000' AND a.ability = 'soil_moisture'",
    )
    .bind(reading)
    .execute(&app.db_pool)
    .await
    .unwrap();

    let stats = app
        .state
        .data_export_sync
        .sync(domain::Id::new(root))
        .await
        .unwrap()
        .unwrap();
    assert!(stats.things_written >= 1);

    let key = format!("irrigation:tree:{tree}");
    let client = reqwest::Client::new();
    let things: Value = client
        .get(format!("{frost}/Things"))
        .query(&[
            ("$filter", format!("properties/ge_key eq '{key}'")),
            (
                "$expand",
                "Locations,Datastreams($expand=Observations)".to_string(),
            ),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(things["value"].as_array().unwrap().len(), 1);
    let thing = &things["value"][0];
    assert_eq!(thing["name"], "Baum E2E");
    assert_eq!(
        thing["Locations"][0]["location"]["coordinates"],
        json!([9.43, 54.79])
    );
    let streams = thing["Datastreams"].as_array().unwrap();
    let stream = |key: String| -> &Value {
        streams
            .iter()
            .find(|s| s["properties"]["ge_key"] == key.as_str())
            .unwrap_or_else(|| panic!("no datastream {key}"))
    };
    let state = stream(format!("state:watering_status:{key}"));
    assert_eq!(state["Observations"].as_array().unwrap().len(), 1);
    assert_eq!(state["Observations"][0]["result"], "moderate");
    for (depth, result) in [(40, 42.0), (80, 71.0)] {
        let raw = stream(format!("raw:{sensor}:soil_moisture:{depth}:{key}"));
        let observations = raw["Observations"].as_array().unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0]["result"].as_f64(), Some(result));
        assert_eq!(
            observations[0]["parameters"]["ge_reading_id"],
            reading.to_string()
        );
    }
    let plausible = &stream(format!("raw:{sensor}:soil_moisture:40:{key}"))["Observations"][0];
    assert!(plausible["parameters"]["quality_issue"].is_null());
    let implausible = &stream(format!("raw:{sensor}:soil_moisture:80:{key}"))["Observations"][0];
    assert_eq!(implausible["parameters"]["quality_issue"], "out of range");
    assert_eq!(streams.len(), 3);

    // A second pass within the interval is not due yet.
    sqlx::query("UPDATE data_export_targets SET last_run_at = now() - interval '1 hour'")
        .execute(&app.db_pool)
        .await
        .unwrap();
    let again = app
        .state
        .data_export_sync
        .sync(domain::Id::new(root))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again.things_written, 0);
    assert_eq!(again.observations_written, 0);
}
