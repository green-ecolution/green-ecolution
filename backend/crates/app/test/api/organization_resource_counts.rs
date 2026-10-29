use serde_json::Value;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::{TestApp, seed_user_with_permissions, spawn_app};
use crate::organizations::ROOT_ORG_ID;

async fn insert_org(app: &TestApp, parent: Uuid, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(parent)
        .bind(name)
        .execute(&app.db_pool)
        .await
        .unwrap();
    id
}

async fn insert_sensor(app: &TestApp, org: Uuid) {
    let model_id = app.ecodrizzler_model_id().await;
    sqlx::query(
        "INSERT INTO sensors (id, activated_at, type, model_id, organization_id)
         VALUES ($1, NOW(), 'lorawan', $2, $3)",
    )
    .bind(format!("eui-{}", Uuid::now_v7()))
    .bind(model_id)
    .bind(org)
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_vehicle(app: &TestApp, org: Uuid, archived: bool) {
    let archived_at = if archived { "NOW()" } else { "NULL" };
    sqlx::query(&format!(
        "INSERT INTO vehicles (id, number_plate, description, type, availability, driving_license,
                               model, width, height, length, weight, organization_id, archived_at)
         VALUES ($1, $2, 'Testfahrzeug', 'transporter', 'available', 'B',
                 'VW Transporter', 1.9, 1.99, 5.3, 2.0, $3, {archived_at})"
    ))
    .bind(Uuid::now_v7())
    .bind(format!("FL-{}", &Uuid::new_v4().simple().to_string()[..8]))
    .bind(org)
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_plan(app: &TestApp, org: Uuid) {
    sqlx::query(
        "INSERT INTO operation_plans (id, module_key, date, description, status, organization_id)
         VALUES ($1, 'irrigation', '2026-05-01', '', 'planned', $2)",
    )
    .bind(Uuid::now_v7())
    .bind(org)
    .execute(&app.db_pool)
    .await
    .unwrap();
}

fn entry_for(body: &Value, org: Uuid) -> &Value {
    body.as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["organization_id"] == org.to_string())
        .unwrap_or_else(|| panic!("no entry for {org}"))
}

#[tokio::test]
async fn counts_cover_the_whole_subtree_of_each_organization() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tbz = insert_org(&app, root, "TBZ").await;
    let firma_a = insert_org(&app, tbz, "Firma A").await;
    let andere_stadt = insert_org(&app, root, "Andere Stadt").await;

    insert_sensor(&app, firma_a).await;
    insert_vehicle(&app, tbz, false).await;
    insert_vehicle(&app, firma_a, true).await;
    insert_plan(&app, firma_a).await;
    insert_sensor(&app, andere_stadt).await;
    insert_vehicle(&app, andere_stadt, false).await;
    insert_plan(&app, andere_stadt).await;

    let response = app.get("/api/v1/organizations/resource-counts").await;
    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.unwrap();

    let tbz_entry = entry_for(&body, tbz);
    assert_eq!(tbz_entry["sensor_count"], 1);
    assert_eq!(
        tbz_entry["vehicle_count"], 1,
        "archived vehicles do not count"
    );
    assert_eq!(tbz_entry["plan_count"], 1);

    let firma_a_entry = entry_for(&body, firma_a);
    assert_eq!(firma_a_entry["sensor_count"], 1);
    assert_eq!(firma_a_entry["vehicle_count"], 0);
    assert_eq!(firma_a_entry["plan_count"], 1);

    let andere_stadt_entry = entry_for(&body, andere_stadt);
    assert_eq!(andere_stadt_entry["sensor_count"], 1);
    assert_eq!(andere_stadt_entry["vehicle_count"], 1);
    assert_eq!(andere_stadt_entry["plan_count"], 1);

    let root_entry = entry_for(&body, root);
    assert_eq!(root_entry["sensor_count"], 2);
    assert_eq!(root_entry["vehicle_count"], 2);
    assert_eq!(root_entry["plan_count"], 2);
}

#[tokio::test]
async fn only_visible_organizations_are_listed() {
    let (harness, app) = spawn_with_auth().await;
    let (own_org, token) =
        seed_user_with_permissions(&harness, &app, "Firma X", &["organization:read"]).await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let other = insert_org(&app, root, "Firma Y").await;

    let response = app
        .get_with_bearer("/api/v1/organizations/resource-counts", &token)
        .await;
    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.unwrap();
    let ids: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["organization_id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&own_org.to_string().as_str()));
    assert!(!ids.contains(&other.to_string().as_str()));
    assert!(!ids.contains(&ROOT_ORG_ID));
}

#[tokio::test]
async fn requires_organization_read() {
    let (harness, app) = spawn_with_auth().await;
    let (_org, token) =
        seed_user_with_permissions(&harness, &app, "Firma Z", &["vehicle:read"]).await;

    let response = app
        .get_with_bearer("/api/v1/organizations/resource-counts", &token)
        .await;
    assert_eq!(response.status().as_u16(), 403);
}
