use serde_json::{Value, json};
use uuid::Uuid;

use crate::{helpers::spawn_app_with_data_export, organizations::ROOT_ORG_ID};

/// Needs a running FROST (`just frost-up`); `just test-data-export` sets the URL.
#[tokio::test]
#[ignore]
async fn a_tree_reaches_the_local_frost_server_with_its_watering_status() {
    let frost = std::env::var("GE_TEST_FROST_URL").expect("GE_TEST_FROST_URL");
    let app = spawn_app_with_data_export(&["localhost"]).await;
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
    let thing = &things["value"][0];
    assert_eq!(thing["name"], "Baum E2E");
    assert_eq!(
        thing["Locations"][0]["location"]["coordinates"],
        json!([9.43, 54.79])
    );
    assert_eq!(
        thing["Datastreams"][0]["Observations"][0]["result"],
        "moderate"
    );

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
