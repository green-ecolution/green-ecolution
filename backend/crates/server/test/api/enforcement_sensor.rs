use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::seed_user_with_permissions;

fn sensor_payload(id: &str, model_id: Uuid, org: Uuid) -> serde_json::Value {
    json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "organization_id": org,
        "lorawan": {
            "serial_number": "SN-ENFORCE",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    })
}

fn tree_payload(number: &str, org: Uuid) -> serde_json::Value {
    json!({
        "species": "Tilia", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": "",
        "organization_id": org,
    })
}

#[tokio::test]
async fn foreign_org_sees_no_sensors_and_gets_404_on_detail() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", &["sensor:read", "sensor:create"])
            .await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["sensor:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-001", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b["data"].as_array().unwrap().len(), 0);

    let detail_b = client
        .get(format!("{}/api/v1/sensors/{sensor_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_a["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn activate_by_user_without_tree_update_yields_403() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Org",
        &[
            "sensor:read",
            "sensor:create",
            "sensor:update",
            "tree:read",
            "tree:create",
        ],
    )
    .await;
    let client = reqwest::Client::new();

    let sensor: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token)
        .json(&sensor_payload("eui-enforce-003", model_id, org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap().to_owned();

    let tree: serde_json::Value = client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token)
        .json(&tree_payload("ENFORCE-003", org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tree_id = tree["id"].as_str().unwrap().to_owned();

    let activate_resp = client
        .post(format!(
            "{}/api/v1/sensors/{sensor_id}/activate",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        activate_resp.status(),
        403,
        "sensor:update alone must not be enough - tree:update is also required"
    );
}

#[tokio::test]
async fn sensor_data_endpoint_is_404_for_foreign_org() {
    let (harness, app) = spawn_with_auth().await;
    let model_id = app.ecodrizzler_model_id().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", &["sensor:read", "sensor:create"])
            .await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["sensor:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/sensors", app.address))
        .bearer_auth(&token_a)
        .json(&sensor_payload("eui-enforce-005", model_id, org_a))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sensor_id = created["id"].as_str().unwrap().to_owned();

    app.ingest_ecodrizzler("eui-enforce-005", 45).await.unwrap();

    let data_resp_a = client
        .get(format!("{}/api/v1/sensors/{sensor_id}/data", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap();
    assert_eq!(data_resp_a.status(), 200);

    let data_resp_b = client
        .get(format!("{}/api/v1/sensors/{sensor_id}/data", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(data_resp_b.status(), 404);
}
