use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::{TestApp, spawn_app};
use crate::organizations::ROOT_ORG_ID;

const SCOPE_HEADER: &str = "X-Organization-Scope";
const VEHICLE_PERMS: &[&str] = &["vehicle:read", "vehicle:create", "vehicle:update"];

struct Tree {
    tbz: Uuid,
    firma_a: Uuid,
    firma_b: Uuid,
}

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

/// `root → tbz → {firma_a, firma_b}`
async fn seed_tree(app: &TestApp) -> Tree {
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tbz = insert_org(app, root, "TBZ").await;
    Tree {
        tbz,
        firma_a: insert_org(app, tbz, "Firma A").await,
        firma_b: insert_org(app, tbz, "Firma B").await,
    }
}

async fn seed_user(harness: &AuthHarness, app: &TestApp, org: Uuid, perms: &[&str]) -> String {
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'Scope-Rolle', $3)",
    )
    .bind(role_id)
    .bind(org)
    .bind(perms.iter().map(|p| p.to_string()).collect::<Vec<_>>())
    .execute(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(org)
        .execute(&app.db_pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&app.db_pool)
        .await
        .unwrap();
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

fn vehicle_payload(plate: &str, org: Option<Uuid>) -> Value {
    let mut payload = json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "capacities": { "water": 5000.0 },
        "model": "MAN TGS",
        "availability": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0,
    });
    if let Some(org) = org {
        payload["organization_id"] = json!(org);
    }
    payload
}

fn request(
    builder: reqwest::RequestBuilder,
    token: Option<&str>,
    scope: Option<&str>,
) -> reqwest::RequestBuilder {
    let builder = match token {
        Some(token) => builder.bearer_auth(token),
        None => builder,
    };
    match scope {
        Some(scope) => builder.header(SCOPE_HEADER, scope),
        None => builder,
    }
}

async fn create_vehicle(app: &TestApp, token: Option<&str>, plate: &str, org: Uuid) -> Uuid {
    let resp = request(
        reqwest::Client::new().post(format!("{}/api/v1/vehicles", app.address)),
        token,
        None,
    )
    .json(&vehicle_payload(plate, Some(org)))
    .send()
    .await
    .unwrap();
    assert_eq!(resp.status(), 201, "seeding vehicle {plate}");
    let body: Value = resp.json().await.unwrap();
    Uuid::parse_str(body["id"].as_str().unwrap()).unwrap()
}

async fn listed_plates(app: &TestApp, token: Option<&str>, scope: Option<&str>) -> Vec<String> {
    let resp = request(
        reqwest::Client::new().get(format!("{}/api/v1/vehicles", app.address)),
        token,
        scope,
    )
    .send()
    .await
    .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let mut plates: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["number_plate"].as_str().unwrap().to_owned())
        .collect();
    plates.sort();
    plates
}

async fn seed_three_vehicles(app: &TestApp, token: Option<&str>, tree: &Tree) -> [Uuid; 3] {
    [
        create_vehicle(app, token, "SCP-TBZ", tree.tbz).await,
        create_vehicle(app, token, "SCP-A", tree.firma_a).await,
        create_vehicle(app, token, "SCP-B", tree.firma_b).await,
    ]
}

#[tokio::test]
async fn list_with_scope_shows_only_subtree() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;
    seed_three_vehicles(&app, Some(&token), &tree).await;

    let scoped = listed_plates(&app, Some(&token), Some(&tree.firma_a.to_string())).await;
    assert_eq!(scoped, vec!["SCP-A"]);

    let unscoped = listed_plates(&app, Some(&token), None).await;
    assert_eq!(unscoped, vec!["SCP-A", "SCP-B", "SCP-TBZ"]);
}

#[tokio::test]
async fn create_without_org_lands_in_scope() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;

    let resp = request(
        reqwest::Client::new().post(format!("{}/api/v1/vehicles", app.address)),
        Some(&token),
        Some(&tree.firma_a.to_string()),
    )
    .json(&vehicle_payload("SCP-NEW", None))
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], json!(tree.firma_a));
}

#[tokio::test]
async fn write_outside_scope_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;
    let [_, _, vehicle_b] = seed_three_vehicles(&app, Some(&token), &tree).await;

    let resp = request(
        reqwest::Client::new().put(format!("{}/api/v1/vehicles/{vehicle_b}", app.address)),
        Some(&token),
        Some(&tree.firma_a.to_string()),
    )
    .json(&vehicle_payload("SCP-B", None))
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn unreachable_scope_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.firma_a, VEHICLE_PERMS).await;

    let resp = request(
        reqwest::Client::new().get(format!("{}/api/v1/vehicles", app.address)),
        Some(&token),
        Some(&tree.tbz.to_string()),
    )
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "organization_scope.invalid");
}

#[tokio::test]
async fn malformed_scope_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;

    let resp = request(
        reqwest::Client::new().get(format!("{}/api/v1/vehicles", app.address)),
        Some(&token),
        Some("abc"),
    )
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "organization_scope.invalid");
}

#[tokio::test]
async fn demo_bypass_respects_scope() {
    let app = spawn_app().await;
    let tree = seed_tree(&app).await;
    seed_three_vehicles(&app, None, &tree).await;

    let scoped = listed_plates(&app, None, Some(&tree.firma_a.to_string())).await;
    assert_eq!(scoped, vec!["SCP-A"]);
}

#[tokio::test]
async fn create_with_org_outside_scope_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;

    let resp = request(
        reqwest::Client::new().post(format!("{}/api/v1/vehicles", app.address)),
        Some(&token),
        Some(&tree.firma_a.to_string()),
    )
    .json(&vehicle_payload("SCP-OUT", Some(tree.firma_b)))
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn module_list_respects_scope() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    sqlx::query(
        "INSERT INTO organization_modules (organization_id, module_key, enabled, set_by) VALUES ($1, 'waste', true, $1)",
    )
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();
    for (number, org) in [("SC-A", tree.firma_a), ("SC-B", tree.firma_b)] {
        sqlx::query(
            "INSERT INTO waste_bins (id, number, latitude, longitude, geometry, volume_liters, depth_cm, organization_id)
             VALUES ($1, $2, 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 1100, 120, $3)",
        )
        .bind(Uuid::now_v7())
        .bind(number)
        .bind(org)
        .execute(&app.db_pool)
        .await
        .unwrap();
    }
    let token = seed_user(&harness, &app, tree.tbz, &["waste_bin:read"]).await;

    let resp = request(
        reqwest::Client::new().get(format!("{}/api/v1/waste-bins", app.address)),
        Some(&token),
        Some(&tree.firma_a.to_string()),
    )
    .send()
    .await
    .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let numbers: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["number"].as_str().unwrap())
        .collect();
    assert_eq!(numbers, vec!["SC-A"]);
}

async fn my_organizations(app: &TestApp, token: &str, scope: Option<&str>) -> reqwest::Response {
    request(
        reqwest::Client::new().get(format!("{}/api/v1/users/me/organizations", app.address)),
        Some(token),
        scope,
    )
    .send()
    .await
    .unwrap()
}

#[tokio::test]
async fn my_organizations_ignores_scope() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, VEHICLE_PERMS).await;

    for scope in [tree.firma_a, Uuid::now_v7()] {
        let resp = my_organizations(&app, &token, Some(&scope.to_string())).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<Value> = resp.json().await.unwrap();
        let parent_of = |id: Uuid| {
            body.iter()
                .find(|o| o["id"] == json!(id))
                .map(|o| o["parent_id"].clone())
        };
        assert_eq!(body.len(), 3);
        assert_eq!(parent_of(tree.tbz), Some(json!(ROOT_ORG_ID)));
        assert_eq!(parent_of(tree.firma_a), Some(json!(tree.tbz)));
        assert_eq!(parent_of(tree.firma_b), Some(json!(tree.tbz)));
        let name = body
            .iter()
            .find(|o| o["id"] == json!(tree.firma_a))
            .unwrap();
        assert_eq!(name["name"], "Firma A");
    }
}

#[tokio::test]
async fn my_organizations_without_organization_read() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.firma_a, &["vehicle:read"]).await;

    let resp = my_organizations(&app, &token, None).await;

    assert_eq!(resp.status(), 200);
    let body: Vec<Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["id"], json!(tree.firma_a));
}

async fn seed_member(app: &TestApp, org: Uuid) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(org)
        .execute(&app.db_pool)
        .await
        .unwrap();
    user_id
}

async fn organization_of(app: &TestApp, user_id: Uuid) -> Uuid {
    sqlx::query_scalar("SELECT organization_id FROM user_profiles WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
}

async fn move_user(
    app: &TestApp,
    token: &str,
    scope: Option<&str>,
    user_id: Uuid,
    org: Uuid,
) -> reqwest::Response {
    request(
        reqwest::Client::new().patch(format!(
            "{}/api/v1/users/{user_id}/organization",
            app.address
        )),
        Some(token),
        scope,
    )
    .json(&json!({ "organization_id": org }))
    .send()
    .await
    .unwrap()
}

#[tokio::test]
async fn moving_a_member_into_the_scope_from_outside_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, &["user:update"]).await;
    let member = seed_member(&app, tree.firma_b).await;

    let resp = move_user(
        &app,
        &token,
        Some(&tree.firma_a.to_string()),
        member,
        tree.firma_a,
    )
    .await;

    assert_eq!(resp.status(), 403);
    assert_eq!(organization_of(&app, member).await, tree.firma_b);
}

#[tokio::test]
async fn moving_a_member_out_of_an_organization_without_rights_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.firma_a, &["user:update"]).await;
    let member = seed_member(&app, tree.firma_b).await;

    let resp = move_user(&app, &token, None, member, tree.firma_a).await;

    assert_eq!(resp.status(), 403);
    assert_eq!(organization_of(&app, member).await, tree.firma_b);
}

#[tokio::test]
async fn moving_a_member_between_organizations_with_rights_in_both() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, &["user:update"]).await;
    let member = seed_member(&app, tree.firma_b).await;
    harness.mock_identity_lookups(&[(member, "member")]).await;

    let resp = move_user(&app, &token, None, member, tree.firma_a).await;

    assert_eq!(resp.status(), 200);
    assert_eq!(organization_of(&app, member).await, tree.firma_a);
}

async fn my_map_view(app: &TestApp, token: &str, scope: Option<&str>) -> reqwest::Response {
    request(
        reqwest::Client::new().get(format!("{}/api/v1/users/me/map-view", app.address)),
        Some(token),
        scope,
    )
    .send()
    .await
    .unwrap()
}

#[tokio::test]
async fn map_view_follows_the_scope() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.tbz, &["setting:update"]).await;
    let hamburg_center = json!([53.55, 9.99]);
    let resp = request(
        reqwest::Client::new().put(format!(
            "{}/api/v1/organizations/{}/settings",
            app.address, tree.firma_a
        )),
        Some(&token),
        None,
    )
    .json(&json!({ "map_view": {
        "center": hamburg_center, "bbox": [53.40, 9.75, 53.70, 10.25],
        "min_zoom": 11, "max_zoom": 19
    } }))
    .send()
    .await
    .unwrap();
    assert_eq!(resp.status(), 200, "storing Firma A's view");

    let scoped = my_map_view(&app, &token, Some(&tree.firma_a.to_string())).await;
    assert_eq!(scoped.status(), 200);
    let scoped: Value = scoped.json().await.unwrap();
    assert_eq!(scoped["center"], hamburg_center);

    let unscoped: Value = my_map_view(&app, &token, None).await.json().await.unwrap();
    assert_ne!(unscoped["center"], hamburg_center);
}

#[tokio::test]
async fn map_view_rejects_an_unreachable_scope() {
    let (harness, app) = spawn_with_auth().await;
    let tree = seed_tree(&app).await;
    let token = seed_user(&harness, &app, tree.firma_a, &[]).await;

    let resp = my_map_view(&app, &token, Some(&tree.firma_b.to_string())).await;

    assert_eq!(resp.status(), 403);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "organization_scope.invalid");
}
