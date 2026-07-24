use serde_json::json;

use crate::helpers::spawn_app;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";

async fn insert_tbz_org(app: &crate::helpers::TestApp) {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap(),
        uuid::Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
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
        "name": "Scope Cluster",
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

#[tokio::test]
async fn create_cluster_with_tree_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;

    // tree lives in TBZ, cluster targets ROOT (default) -> purity violation.
    let tree_resp = app
        .post_json("/api/v1/trees", &tree_payload("SCOPE-C-001", Some(TBZ_ORG)))
        .await;
    assert_eq!(tree_resp.status(), 201);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();

    let resp = app
        .post_json("/api/v1/clusters", &cluster_payload(&[tree_id], None))
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn update_tree_into_cluster_from_foreign_org_conflicts() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;

    let cluster_resp = app
        .post_json("/api/v1/clusters", &cluster_payload(&[], Some(TBZ_ORG)))
        .await;
    assert_eq!(cluster_resp.status(), 201);
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    let tree_resp = app
        .post_json("/api/v1/trees", &tree_payload("SCOPE-C-002", None))
        .await;
    assert_eq!(tree_resp.status(), 201);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();

    let update_body = json!({
        "species": "Quercus robur", "number": "SCOPE-C-002", "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": "",
        "tree_cluster_id": cluster_id,
    });
    let resp = app
        .put_json(&format!("/api/v1/trees/{}", tree_id), &update_body)
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn create_cluster_with_matching_trees_returns_201_with_organization() {
    let app = spawn_app().await;
    insert_tbz_org(&app).await;

    let tree_resp = app
        .post_json("/api/v1/trees", &tree_payload("SCOPE-C-003", Some(TBZ_ORG)))
        .await;
    assert_eq!(tree_resp.status(), 201);
    let tree: serde_json::Value = tree_resp.json().await.unwrap();
    let tree_id = tree["id"].as_str().unwrap();

    let resp = app
        .post_json(
            "/api/v1/clusters",
            &cluster_payload(&[tree_id], Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}
