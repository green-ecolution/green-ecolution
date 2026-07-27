use serde_json::json;

use crate::helpers::spawn_app;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";
const SUB_ORG: &str = "01980000-0000-7000-8000-000000000003";

async fn insert_tbz_and_sub_org(app: &crate::helpers::TestApp) {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap(),
        uuid::Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'TBZ Unter-Org')",
        uuid::Uuid::parse_str(SUB_ORG).unwrap(),
        uuid::Uuid::parse_str(TBZ_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

fn cluster_payload(org: &str) -> serde_json::Value {
    json!({
        "name": "Scope Plan Cluster",
        "address": "Am Testfeld 1",
        "description": "",
        "soil_condition": "Su3",
        "tree_ids": [],
        "organization_id": org,
    })
}

async fn create_transporter(app: &crate::helpers::TestApp) -> serde_json::Value {
    let body = json!({
        "number_plate": "FL-GE 300",
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

fn plan_payload(transporter_id: &str, cluster_ids: &[&str], org: &str) -> serde_json::Value {
    json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Scope Plan",
        "transporter_id": transporter_id,
        "tree_cluster_ids": cluster_ids,
        "user_ids": [],
        "organization_id": org,
    })
}

#[tokio::test]
async fn create_watering_plan_over_cluster_in_org_subtree_returns_201() {
    let app = spawn_app().await;
    insert_tbz_and_sub_org(&app).await;

    // cluster lives in the sub-org of TBZ, plan targets TBZ -> subtree is
    // reachable from the plan's org.
    let cluster_resp = app
        .post_json("/api/v1/clusters", &cluster_payload(SUB_ORG))
        .await;
    assert_eq!(cluster_resp.status(), 201);
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    let transporter = create_transporter(&app).await;
    let transporter_id = transporter["id"].as_str().unwrap();

    let resp = app
        .post_json(
            "/api/v1/watering-plans",
            &plan_payload(transporter_id, &[cluster_id], TBZ_ORG),
        )
        .await;

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}

#[tokio::test]
async fn create_watering_plan_over_cluster_outside_org_subtree_conflicts() {
    let app = spawn_app().await;
    insert_tbz_and_sub_org(&app).await;

    // cluster lives in TBZ, plan targets the sub-org -> TBZ is an ancestor,
    // not reachable from the plan's org.
    let cluster_resp = app
        .post_json("/api/v1/clusters", &cluster_payload(TBZ_ORG))
        .await;
    assert_eq!(cluster_resp.status(), 201);
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    let transporter = create_transporter(&app).await;
    let transporter_id = transporter["id"].as_str().unwrap();

    let resp = app
        .post_json(
            "/api/v1/watering-plans",
            &plan_payload(transporter_id, &[cluster_id], SUB_ORG),
        )
        .await;

    assert_eq!(resp.status(), 409);
}
