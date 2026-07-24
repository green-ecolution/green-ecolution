use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::{TestApp, seed_user_with_permissions};

const ALL_READ_PERMS: &[&str] = &[
    "tree:read",
    "tree_cluster:read",
    "sensor:read",
    "watering_plan:read",
    "vehicle:read",
];

const ALL_READ_AND_TREE_CREATE_PERMS: &[&str] = &[
    "tree:read",
    "tree:create",
    "tree_cluster:read",
    "sensor:read",
    "watering_plan:read",
    "vehicle:read",
];

#[tokio::test]
async fn tree_only_reader_sees_own_tree_but_no_other_resource_counts() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", &["tree:read", "tree:create"]).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token_a)
        .json(&json!({
            "species": "Tilia", "number": "EVAL-1", "planting_year": 2024,
            "latitude": 54.79, "longitude": 9.44, "description": "",
            "organization_id": org_a
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = client
        .get(format!("{}/api/v1/evaluation", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body["tree_count"], 1);
    assert_eq!(body["treecluster_count"], 0);
    assert_eq!(body["sensor_count"], 0);
    assert_eq!(body["watering_plan_count"], 0);
    assert!(body["vehicle_evaluation"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn foreign_org_with_no_resources_sees_all_zero_counts() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", ALL_READ_AND_TREE_CREATE_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", ALL_READ_PERMS).await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/api/v1/trees", app.address))
        .bearer_auth(&token_a)
        .json(&json!({
            "species": "Tilia", "number": "EVAL-2", "planting_year": 2024,
            "latitude": 54.79, "longitude": 9.44, "description": "",
            "organization_id": org_a
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = client
        .get(format!("{}/api/v1/evaluation", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body["tree_count"], 0);
    assert_eq!(body["treecluster_count"], 0);
    assert_eq!(body["sensor_count"], 0);
    assert_eq!(body["watering_plan_count"], 0);
    assert_eq!(body["total_water_consumption"], 0);
    assert!(body["region_evaluation"].as_array().unwrap().is_empty());
    assert!(body["vehicle_evaluation"].as_array().unwrap().is_empty());
}

const CLUSTER_AND_PLAN_CREATE_PERMS: &[&str] = &[
    "tree_cluster:read",
    "tree_cluster:create",
    "watering_plan:read",
    "watering_plan:create",
    "vehicle:read",
    "vehicle:create",
];

async fn create_cluster(
    client: &reqwest::Client,
    app: &TestApp,
    token: &str,
    org: Uuid,
) -> serde_json::Value {
    client
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(token)
        .json(&json!({
            "name": "Eval Cluster",
            "address": "Am Testfeld 2",
            "description": "",
            "soil_condition": "Su3",
            "tree_ids": [],
            "organization_id": org,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn create_plan_with_cluster(
    client: &reqwest::Client,
    app: &TestApp,
    token: &str,
    org: Uuid,
    cluster_id: &str,
) -> serde_json::Value {
    let transporter: serde_json::Value = client
        .post(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(token)
        .json(&json!({
            "number_plate": format!("FL-GE {}", &Uuid::new_v4().to_string()[..8]),
            "description": "Giesswagen",
            "water_capacity": 5000.0,
            "model": "MAN TGS",
            "status": "available",
            "type": "transporter",
            "driving_license": "C",
            "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let transporter_id = transporter["id"].as_str().unwrap();

    client
        .post(format!("{}/api/v1/watering-plans", app.address))
        .bearer_auth(token)
        .json(&json!({
            "date": "2026-05-01T08:00:00Z",
            "description": "Eval Plan",
            "transporter_id": transporter_id,
            "tree_cluster_ids": [cluster_id],
            "user_ids": [Uuid::new_v4().to_string()],
            "organization_id": org,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn foreign_org_sees_no_water_consumption_or_user_plan_count() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) =
        seed_user_with_permissions(&harness, &app, "Org A", CLUSTER_AND_PLAN_CREATE_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", ALL_READ_PERMS).await;
    let client = reqwest::Client::new();

    let cluster = create_cluster(&client, &app, &token_a, org_a).await;
    let cluster_id = cluster["id"].as_str().unwrap();
    let plan = create_plan_with_cluster(&client, &app, &token_a, org_a, cluster_id).await;
    let plan_id = plan["id"].as_str().unwrap();

    sqlx::query!(
        "UPDATE tree_cluster_watering_plans SET consumed_water = 250.0 WHERE watering_plan_id = $1",
        Uuid::parse_str(plan_id).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let body_b: serde_json::Value = client
        .get(format!("{}/api/v1/evaluation", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        body_b["total_water_consumption"], 0,
        "a foreign org must not see another tenant's water consumption"
    );
    assert_eq!(
        body_b["user_watering_plan_count"], 0,
        "a foreign org must not see another tenant's watering-plan user assignments"
    );

    let body_a: serde_json::Value = client
        .get(format!("{}/api/v1/evaluation", app.address))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body_a["total_water_consumption"], 250);
    assert_eq!(body_a["user_watering_plan_count"], 1);
}
