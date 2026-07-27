use serde_json::json;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method, matchers::path};

use crate::auth_helpers::AuthHarness;
use crate::helpers::{TestApp, seed_user_with_permissions, spawn_app_with_routing_and_auth};

const VEHICLE_PERMS: &[&str] = &["vehicle:read", "vehicle:create", "vehicle:update"];
const PLAN_PERMS: &[&str] = &[
    "watering_plan:read",
    "watering_plan:create",
    "watering_plan:update",
];

fn vehicle_payload(plate: &str, org: Uuid) -> serde_json::Value {
    json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0,
        "organization_id": org,
    })
}

async fn create_vehicle(
    client: &reqwest::Client,
    app: &TestApp,
    token: &str,
    plate: &str,
    org: Uuid,
) -> serde_json::Value {
    client
        .post(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(token)
        .json(&vehicle_payload(plate, org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// Adds a reader-only user (a single permission) in an already-seeded org.
async fn seed_reader_in_org(
    harness: &AuthHarness,
    app: &TestApp,
    org: Uuid,
    permission: &str,
) -> String {
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Reader', $2)
           RETURNING id"#,
        org,
        &[permission.to_string()],
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)"#,
        user_id,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

async fn spawn() -> (AuthHarness, TestApp) {
    let harness = AuthHarness::start().await;
    let mock_streamlet = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_streamlet)
        .await;
    let app =
        spawn_app_with_routing_and_auth(&mock_streamlet.uri(), harness.auth_settings(true)).await;
    (harness, app)
}

#[tokio::test]
async fn foreign_org_sees_no_vehicles_and_gets_404_on_detail() {
    let (harness, app) = spawn().await;
    let (org_a, token_a) = seed_user_with_permissions(&harness, &app, "Org A", VEHICLE_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["vehicle:read"]).await;
    let client = reqwest::Client::new();

    let created = create_vehicle(&client, &app, &token_a, "ENF-VEH-001", org_a).await;
    let vehicle_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b["data"].as_array().unwrap().len(), 0);

    let detail_b = client
        .get(format!("{}/api/v1/vehicles/{vehicle_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 404);

    let by_plate_b = client
        .get(format!("{}/api/v1/vehicles/plate/ENF-VEH-001", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(by_plate_b.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/vehicles", app.address))
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
async fn update_without_update_permission_yields_403() {
    let (harness, app) = spawn().await;
    let (org, token_admin) = seed_user_with_permissions(&harness, &app, "Org", VEHICLE_PERMS).await;
    let client = reqwest::Client::new();

    let created = create_vehicle(&client, &app, &token_admin, "ENF-VEH-002", org).await;
    let vehicle_id = created["id"].as_str().unwrap().to_owned();

    let token_reader = seed_reader_in_org(&harness, &app, org, "vehicle:read").await;

    let update_resp = client
        .put(format!("{}/api/v1/vehicles/{vehicle_id}", app.address))
        .bearer_auth(&token_reader)
        .json(&json!({
            "number_plate": "ENF-VEH-002",
            "description": "Geaendert",
            "water_capacity": 6000.0,
            "model": "MAN TGS",
            "status": "available",
            "type": "transporter",
            "driving_license": "C",
            "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        update_resp.status(),
        403,
        "vehicle:read alone must not allow an update"
    );
}

#[tokio::test]
async fn archive_by_foreign_org_yields_404() {
    let (harness, app) = spawn().await;
    let (org_a, token_a) = seed_user_with_permissions(&harness, &app, "Org A", VEHICLE_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", VEHICLE_PERMS).await;
    let client = reqwest::Client::new();

    let created = create_vehicle(&client, &app, &token_a, "ENF-VEH-003", org_a).await;
    let vehicle_id = created["id"].as_str().unwrap().to_owned();

    let archive_resp = client
        .post(format!(
            "{}/api/v1/vehicles/archived/{vehicle_id}",
            app.address
        ))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(archive_resp.status(), 404);
}

#[tokio::test]
async fn foreign_org_sees_no_start_points() {
    let (harness, app) = spawn().await;
    let (org_a, token_a) = seed_user_with_permissions(&harness, &app, "Org A", PLAN_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["watering_plan:read"]).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/routing/start-points", app.address))
        .bearer_auth(&token_a)
        .json(&json!({
            "name": "Depot A",
            "lat": 54.7,
            "lon": 9.4,
            "watering_point": false,
            "organization_id": org_a,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sp_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/routing/start-points", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        list_b.as_array().unwrap().len(),
        0,
        "foreign org must not see any depots, including seeded root ones"
    );

    let set_default_resp = client
        .post(format!(
            "{}/api/v1/routing/start-points/{sp_id}/default",
            app.address
        ))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(set_default_resp.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/routing/start-points", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        list_a
            .as_array()
            .unwrap()
            .iter()
            .any(|sp| sp["id"].as_str() == Some(sp_id.as_str())),
        "Org A must see its own depot"
    );
}

#[tokio::test]
async fn update_start_point_without_update_permission_yields_403() {
    let (harness, app) = spawn().await;
    let (org, token_admin) = seed_user_with_permissions(&harness, &app, "Org", PLAN_PERMS).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/routing/start-points", app.address))
        .bearer_auth(&token_admin)
        .json(&json!({
            "name": "Depot",
            "lat": 54.7,
            "lon": 9.4,
            "watering_point": false,
            "organization_id": org,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sp_id = created["id"].as_str().unwrap().to_owned();

    let token_reader = seed_reader_in_org(&harness, &app, org, "watering_plan:read").await;

    let update_resp = client
        .put(format!(
            "{}/api/v1/routing/start-points/{sp_id}",
            app.address
        ))
        .bearer_auth(&token_reader)
        .json(&json!({
            "name": "Depot Neu",
            "lat": 54.7,
            "lon": 9.4,
            "watering_point": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        update_resp.status(),
        403,
        "watering_plan:read alone must not allow a start point update"
    );
}
