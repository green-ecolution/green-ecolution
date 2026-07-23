use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::spawn_app;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";
const SUB_ORG: &str = "01980000-0000-7000-8000-000000000003";

/// Returns the sibling org's id (parented directly at root, alongside TBZ).
async fn insert_org_tree(app: &crate::helpers::TestApp) -> Uuid {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ')",
        Uuid::parse_str(TBZ_ORG).unwrap(),
        Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ Unter-Org')",
        Uuid::parse_str(SUB_ORG).unwrap(),
        Uuid::parse_str(TBZ_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(ROOT_ORG).unwrap(),
        "Geschwister-Org",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

fn tree_payload(number: &str, org: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "species": "Quercus robur", "number": number, "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": ""
    });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

fn cluster_payload(tree_ids: &[&str], org: Option<&str>) -> serde_json::Value {
    let mut p = json!({
        "name": "Sharing Cluster",
        "address": "Am Testfeld 1",
        "description": "",
        "soil_condition": "Su3",
        "tree_ids": tree_ids,
    });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

async fn create_tree(app: &crate::helpers::TestApp, number: &str, org: &str) -> String {
    let resp = app
        .post_json("/api/v1/trees", &tree_payload(number, Some(org)))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_owned()
}

async fn create_cluster(app: &crate::helpers::TestApp, tree_ids: &[&str], org: &str) -> String {
    let resp = app
        .post_json("/api/v1/clusters", &cluster_payload(tree_ids, Some(org)))
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn share_cluster_with_sub_org_cascades_to_its_trees() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let tree_id = create_tree(&app, "SHARE-A-001", TBZ_ORG).await;
    let cluster_id = create_cluster(&app, &[&tree_id], TBZ_ORG).await;

    let resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let cluster_resp = app.get(&format!("/api/v1/clusters/{cluster_id}")).await;
    assert_eq!(cluster_resp.status(), 200);
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    assert_eq!(cluster["shared_with"], json!([SUB_ORG]));

    let tree_resp = app.get(&format!("/api/v1/trees/{tree_id}")).await;
    assert_eq!(tree_resp.status(), 200);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    assert_eq!(tree["shared_with"], json!([SUB_ORG]));
}

#[tokio::test]
async fn share_cluster_with_sibling_org_returns_422() {
    let app = spawn_app().await;
    let sibling_org = insert_org_tree(&app).await;

    let cluster_id = create_cluster(&app, &[], TBZ_ORG).await;

    let resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": sibling_org }),
        )
        .await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn share_cluster_with_owning_org_itself_returns_422() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let cluster_id = create_cluster(&app, &[], TBZ_ORG).await;

    let resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": TBZ_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn share_tree_in_cluster_returns_409() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let tree_id = create_tree(&app, "SHARE-D-001", TBZ_ORG).await;
    let _cluster_id = create_cluster(&app, &[&tree_id], TBZ_ORG).await;

    let resp = app
        .post_json(
            &format!("/api/v1/trees/{tree_id}/shares"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn revoke_share_removes_it_from_the_view() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    let cluster_id = create_cluster(&app, &[], TBZ_ORG).await;

    let resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let resp = app
        .delete(&format!("/api/v1/clusters/{cluster_id}/shares/{SUB_ORG}"))
        .await;
    assert_eq!(resp.status(), 204);

    let cluster_resp = app.get(&format!("/api/v1/clusters/{cluster_id}")).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    assert_eq!(cluster["shared_with"], json!([]));
}

async fn create_transporter(app: &crate::helpers::TestApp) -> serde_json::Value {
    let body = json!({
        "number_plate": "FL-GE 301",
        "description": "Giesswagen",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
    });
    let resp = app.post_json("/api/v1/vehicles", &body).await;
    assert_eq!(resp.status(), 201);
    resp.json().await.unwrap()
}

#[tokio::test]
async fn watering_plan_of_sub_org_can_use_a_cluster_shared_with_it() {
    let app = spawn_app().await;
    insert_org_tree(&app).await;

    // cluster is owned by TBZ (not in the sub-org's own subtree) and shared
    // explicitly with SUB_ORG -> extended accessibility should allow it.
    let cluster_id = create_cluster(&app, &[], TBZ_ORG).await;
    let resp = app
        .post_json(
            &format!("/api/v1/clusters/{cluster_id}/shares"),
            &json!({ "organization_id": SUB_ORG }),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let transporter = create_transporter(&app).await;
    let transporter_id = transporter["id"].as_str().unwrap();

    let plan_body = json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Shared Cluster Plan",
        "transporter_id": transporter_id,
        "tree_cluster_ids": [cluster_id],
        "user_ids": [],
        "organization_id": SUB_ORG,
    });
    let resp = app.post_json("/api/v1/watering-plans", &plan_body).await;
    assert_eq!(resp.status(), 201);
}

/// Seeds an admin-copy role (full permission set) in `org_id` and a user
/// holding it, mirroring `authorization.rs::seed_admin_in_new_org` but
/// reusing an org that already exists (`insert_org_tree` created it).
async fn seed_admin(
    harness: &crate::auth_helpers::AuthHarness,
    app: &crate::helpers::TestApp,
    org_id: Uuid,
) -> String {
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           SELECT gen_random_uuid(), $1, 'Org-Admin', permissions FROM roles WHERE id = '01980000-0000-7000-8000-0000000000a1'
           RETURNING id"#,
        org_id
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
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

#[tokio::test]
async fn sharing_a_cluster_requires_update_permission_in_the_owning_org() {
    let (harness, app) = spawn_with_auth().await;
    let _ = insert_org_tree(&app).await;

    let admin_token = seed_admin(&harness, &app, Uuid::parse_str(TBZ_ORG).unwrap()).await;
    let cluster_resp = reqwest::Client::new()
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(&admin_token)
        .json(&cluster_payload(&[], Some(TBZ_ORG)))
        .send()
        .await
        .unwrap();
    assert_eq!(cluster_resp.status(), 201);
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // user without any grants
    let token = harness.sign_token(json!({ "sub": Uuid::now_v7().to_string() }));
    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/shares",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "organization_id": SUB_ORG }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
