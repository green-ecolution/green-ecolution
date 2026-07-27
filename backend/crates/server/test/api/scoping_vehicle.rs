use serde_json::json;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::spawn_app;

const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";

fn vehicle_payload(plate: &str, org: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2,
        "width": 2.5,
        "length": 8.0,
        "weight": 12000.0
    });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

#[tokio::test]
async fn create_vehicle_stores_explicit_organization() {
    let app = spawn_app().await;
    // demo bypass: unrestricted, explicit org wins
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, '01980000-0000-7000-8000-000000000001', 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let resp = app
        .post_json(
            "/api/v1/vehicles",
            &vehicle_payload("SCOPE-001", Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}

#[tokio::test]
async fn create_vehicle_defaults_to_root_in_demo_mode() {
    let app = spawn_app().await;
    let resp = app
        .post_json("/api/v1/vehicles", &vehicle_payload("SCOPE-002", None))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["organization_id"],
        "01980000-0000-7000-8000-000000000001"
    );
}

#[tokio::test]
async fn create_vehicle_requires_create_permission_in_target_org() {
    let (harness, app) = spawn_with_auth().await;
    // user without any grants
    let token = harness.sign_token(json!({ "sub": uuid::Uuid::now_v7().to_string() }));
    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(&token)
        .json(&vehicle_payload(
            "SCOPE-003",
            Some("01980000-0000-7000-8000-000000000001"),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
