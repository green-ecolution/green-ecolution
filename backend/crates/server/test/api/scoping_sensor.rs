use serde_json::json;
use uuid::Uuid;

use crate::helpers::{TestApp, spawn_app};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";

async fn insert_tbz_org(app: &TestApp) {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap(),
        uuid::Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

fn sensor_payload(id: &str, model_id: Uuid, org: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "id": id,
        "sensor_type": "lorawan",
        "model_id": model_id,
        "lorawan": {
            "serial_number": "SN-SCOPE",
            "dev_eui": "a81758fffe0c3b52",
            "app_eui": "70b3d57ed00abcd1",
            "app_key": "00112233445566778899aabbccddeeff"
        }
    });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

async fn insert_tree(app: &TestApp, number: &str, organization_id: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description, organization_id)
        VALUES ($1, 2020, 'Eiche', $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), 'Test', $5)"#,
        id,
        number,
        54.79_f64,
        9.45_f64,
        uuid::Uuid::parse_str(organization_id).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn create_sensor_response_carries_organization_id() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    let resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-001", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}

#[tokio::test]
async fn activate_sensor_on_tree_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-002", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    // tree lives in ROOT, sensor lives in TBZ -> org mismatch.
    let tree_id = insert_tree(&app, "SCOPE-S-002", ROOT_ORG).await;

    let resp = app
        .post_json(
            "/api/v1/sensors/eui-scope-002/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn activate_sensor_on_tree_from_same_org_returns_200() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-003", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    let tree_id = insert_tree(&app, "SCOPE-S-003", TBZ_ORG).await;

    let resp = app
        .post_json(
            "/api/v1/sensors/eui-scope-003/activate",
            &json!({ "tree_id": tree_id }),
        )
        .await;
    assert_eq!(resp.status(), 200);
}

fn tree_update_payload(number: &str, sensor_id: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "species": "Quercus robur", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": ""
    });
    if let Some(sensor_id) = sensor_id {
        p["sensor_id"] = json!(sensor_id);
    }
    p
}

fn tree_create_payload(
    number: &str,
    organization_id: &str,
    sensor_id: Option<&str>,
) -> serde_json::Value {
    let mut p = tree_update_payload(number, sensor_id);
    p["organization_id"] = json!(organization_id);
    p
}

#[tokio::test]
async fn update_tree_with_sensor_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    // sensor lives in TBZ, tree lives in ROOT -> org mismatch.
    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-005", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    let tree_id = insert_tree(&app, "SCOPE-S-005", ROOT_ORG).await;

    let resp = app
        .put_json(
            &format!("/api/v1/trees/{tree_id}"),
            &tree_update_payload("SCOPE-S-005", Some("eui-scope-005")),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn create_tree_with_sensor_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    // sensor lives in TBZ, tree is created in ROOT -> org mismatch.
    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-006", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    let resp = app
        .post_json(
            "/api/v1/trees",
            &tree_create_payload("SCOPE-S-006", ROOT_ORG, Some("eui-scope-006")),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn update_tree_with_sensor_from_same_org_returns_200() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-007", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    let tree_id = insert_tree(&app, "SCOPE-S-007", TBZ_ORG).await;

    let resp = app
        .put_json(
            &format!("/api/v1/trees/{tree_id}"),
            &tree_update_payload("SCOPE-S-007", Some("eui-scope-007")),
        )
        .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn reassign_sensor_to_tree_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;
    let model_id = app.ecodrizzler_model_id().await;

    let sensor_resp = app
        .post_json(
            "/api/v1/sensors",
            &sensor_payload("eui-scope-004", model_id, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(sensor_resp.status(), 201);

    let own_tree_id = insert_tree(&app, "SCOPE-S-004A", TBZ_ORG).await;
    let activate_resp = app
        .post_json(
            "/api/v1/sensors/eui-scope-004/activate",
            &json!({ "tree_id": own_tree_id }),
        )
        .await;
    assert_eq!(activate_resp.status(), 200);

    let foreign_tree_id = insert_tree(&app, "SCOPE-S-004B", ROOT_ORG).await;
    let resp = app
        .put_json(
            "/api/v1/sensors/eui-scope-004/tree",
            &json!({ "tree_id": foreign_tree_id }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}
