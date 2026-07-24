use serde_json::json;
use uuid::Uuid;
use wiremock::MockServer;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::{TestApp, seed_user_with_permissions, spawn_app_with_routing_and_auth};

/// Seeds an org below `parent` (not necessarily ROOT), a role with exactly
/// `permissions`, and a user holding it. Needed on top of
/// `seed_user_with_permissions` (which always parents under ROOT) so tests
/// can build a genuine subtree (TBZ -> sub-org).
async fn seed_child_org_with_permissions(
    harness: &AuthHarness,
    app: &TestApp,
    parent: Uuid,
    org_name: &str,
    permissions: &[&str],
) -> (Uuid, String) {
    let org_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1, $2) RETURNING id"#,
        parent,
        org_name,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Test-Rolle', $2)
           RETURNING id"#,
        org_id,
        &permissions,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org_id
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
    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    (org_id, token)
}

fn transporter_payload() -> serde_json::Value {
    json!({
        "number_plate": format!("FL-GE {}", &Uuid::new_v4().to_string()[..8]),
        "description": "Giesswagen",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
    })
}

fn plan_payload(transporter_id: &str, org: Uuid) -> serde_json::Value {
    json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Enforcement Plan",
        "transporter_id": transporter_id,
        "tree_cluster_ids": [],
        "user_ids": [],
        "organization_id": org,
    })
}

const PLAN_PERMS: &[&str] = &[
    "watering_plan:read",
    "watering_plan:create",
    "watering_plan:update",
    "vehicle:read",
    "vehicle:create",
];

async fn create_plan(
    client: &reqwest::Client,
    app: &TestApp,
    token: &str,
    org: Uuid,
) -> serde_json::Value {
    let transporter: serde_json::Value = client
        .post(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(token)
        .json(&transporter_payload())
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
        .json(&plan_payload(transporter_id, org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn foreign_org_sees_no_plans_and_gets_404_on_detail() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) = seed_user_with_permissions(&harness, &app, "Org A", PLAN_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["watering_plan:read"]).await;
    let client = reqwest::Client::new();

    let created = create_plan(&client, &app, &token_a, org_a).await;
    let plan_id = created["id"].as_str().unwrap().to_owned();

    let list_b: serde_json::Value = client
        .get(format!("{}/api/v1/watering-plans", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b["data"].as_array().unwrap().len(), 0);

    let detail_b = client
        .get(format!("{}/api/v1/watering-plans/{plan_id}", app.address))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(detail_b.status(), 404);

    let list_a: serde_json::Value = client
        .get(format!("{}/api/v1/watering-plans", app.address))
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
async fn subtree_visibility_is_one_directional() {
    let (harness, app) = spawn_with_auth().await;
    let (org_tbz, token_tbz) = seed_user_with_permissions(&harness, &app, "TBZ", PLAN_PERMS).await;
    let (org_sub, token_sub) =
        seed_child_org_with_permissions(&harness, &app, org_tbz, "TBZ Unter-Org", PLAN_PERMS).await;
    let client = reqwest::Client::new();

    let sub_plan = create_plan(&client, &app, &token_sub, org_sub).await;
    let sub_plan_id = sub_plan["id"].as_str().unwrap().to_owned();

    let tbz_plan = create_plan(&client, &app, &token_tbz, org_tbz).await;
    let tbz_plan_id = tbz_plan["id"].as_str().unwrap().to_owned();

    // TBZ's grant covers its own subtree, so it must see the sub-org's plan.
    let detail_tbz_sees_sub = client
        .get(format!(
            "{}/api/v1/watering-plans/{sub_plan_id}",
            app.address
        ))
        .bearer_auth(&token_tbz)
        .send()
        .await
        .unwrap();
    assert_eq!(
        detail_tbz_sees_sub.status(),
        200,
        "TBZ must see a plan owned by its sub-org"
    );

    // The sub-org's grant never extends upwards to its parent.
    let detail_sub_sees_tbz = client
        .get(format!(
            "{}/api/v1/watering-plans/{tbz_plan_id}",
            app.address
        ))
        .bearer_auth(&token_sub)
        .send()
        .await
        .unwrap();
    assert_eq!(
        detail_sub_sees_tbz.status(),
        404,
        "the sub-org must not see a plan owned by its parent TBZ"
    );

    let list_tbz: serde_json::Value = client
        .get(format!("{}/api/v1/watering-plans", app.address))
        .bearer_auth(&token_tbz)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        list_tbz["data"].as_array().unwrap().len(),
        2,
        "TBZ must see both its own and its sub-org's plan"
    );

    let list_sub: serde_json::Value = client
        .get(format!("{}/api/v1/watering-plans", app.address))
        .bearer_auth(&token_sub)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        list_sub["data"].as_array().unwrap().len(),
        1,
        "the sub-org must see only its own plan"
    );
}

#[tokio::test]
async fn status_transition_without_update_permission_yields_403() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token_admin) = seed_user_with_permissions(&harness, &app, "Org", PLAN_PERMS).await;
    let client = reqwest::Client::new();

    let created = create_plan(&client, &app, &token_admin, org).await;
    let plan_id = created["id"].as_str().unwrap().to_owned();
    let transporter_id = created["transporter"]["id"].as_str().unwrap().to_owned();

    // Reader-only role in the owning org: can see the plan, but must not be
    // able to change its status.
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Reader', ARRAY['watering_plan:read'])
           RETURNING id"#,
        org
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let reader_user = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        reader_user,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)"#,
        reader_user,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    let token_reader_only = harness.sign_token(json!({ "sub": reader_user.to_string() }));

    let update_resp = client
        .put(format!("{}/api/v1/watering-plans/{plan_id}", app.address))
        .bearer_auth(&token_reader_only)
        .json(&json!({
            "date": "2026-05-01T08:00:00Z",
            "description": "Enforcement Plan",
            "status": "active",
            "transporter_id": transporter_id,
            "tree_cluster_ids": [],
            "user_ids": [],
            "cancellation_note": "",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        update_resp.status(),
        403,
        "watering_plan:read alone must not allow a status transition"
    );
}

#[tokio::test]
async fn route_and_gpx_endpoints_are_404_for_foreign_org() {
    // Routing must be enabled here: with it disabled the feature-flag gate
    // (checked before visibility, see `route_endpoint_returns_503_when_routing_disabled`
    // in routing.rs) would return 503 for every caller and never exercise the
    // visibility check this test is about.
    let harness = AuthHarness::start().await;
    let streamlet = MockServer::start().await;
    let app = spawn_app_with_routing_and_auth(&streamlet.uri(), harness.auth_settings(true)).await;
    let (org_a, token_a) = seed_user_with_permissions(&harness, &app, "Org A", PLAN_PERMS).await;
    let (_org_b, token_b) =
        seed_user_with_permissions(&harness, &app, "Org B", &["watering_plan:read"]).await;
    let client = reqwest::Client::new();

    let created = create_plan(&client, &app, &token_a, org_a).await;
    let plan_id = created["id"].as_str().unwrap().to_owned();

    let route_resp = client
        .get(format!(
            "{}/api/v1/watering-plans/{plan_id}/route",
            app.address
        ))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(route_resp.status(), 404);

    let gpx_resp = client
        .get(format!(
            "{}/api/v1/watering-plans/{plan_id}/route/gpx",
            app.address
        ))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(gpx_resp.status(), 404);
}
