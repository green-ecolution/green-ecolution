use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::{AuthHarness, spawn_with_auth};
use crate::helpers::TestApp;

pub(crate) const ORG: &str = "01980000-0000-7000-8000-000000000001";

pub(crate) const DISPATCH: &[&str] = &[
    "watering_plan:read",
    "watering_plan:create",
    "watering_plan:update",
    "vehicle:read",
    "vehicle:create",
    "tree:read",
    "tree:create",
    "tree_cluster:read",
    "tree_cluster:create",
];
pub(crate) const DRIVER: &[&str] = &["watering_plan:read"];

pub(crate) async fn seed_user_in_org(
    harness: &AuthHarness,
    app: &TestApp,
    perms: &[&str],
) -> (String, Uuid) {
    let org = Uuid::parse_str(ORG).unwrap();
    let perms: Vec<String> = perms.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING id",
    )
    .bind(org)
    .bind(format!("Rolle {}", Uuid::now_v7()))
    .bind(&perms)
    .fetch_one(&app.db_pool)
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
    (
        harness.sign_token(json!({ "sub": user_id.to_string() })),
        user_id,
    )
}

/// Creates a planned plan with one stop (a cluster with one tree) in `ORG`,
/// assigned to `assignee`, as the dispatcher. Returns (plan_id, cluster_id).
pub(crate) async fn planned_plan(
    app: &TestApp,
    dispatcher: &str,
    assignee: Option<Uuid>,
    date: &str,
) -> (String, String) {
    let transporter = app
        .post_json_with_bearer(
            "/api/v1/vehicles",
            &json!({
                "number_plate": format!("FL GE {}", &Uuid::new_v4().simple().to_string()[..4]),
                "description": "Giesswagen",
                "capacities": { "water": 5000.0 },
                "model": "MAN TGS",
                "availability": "available",
                "type": "transporter",
                "driving_license": "C",
                "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0,
                "organization_id": ORG,
            }),
            dispatcher,
        )
        .await;
    assert_eq!(transporter.status().as_u16(), 201, "vehicle setup failed");
    let transporter: serde_json::Value = transporter.json().await.unwrap();

    let tree_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description, organization_id)
        VALUES ($1, 2020, 'Eiche', 'EX-' || substr($1::text, 25), 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'Execution', $2)"#,
    )
    .bind(tree_id)
    .bind(Uuid::parse_str(ORG).unwrap())
    .execute(&app.db_pool)
    .await
    .unwrap();
    let cluster = app
        .post_json_with_bearer(
            "/api/v1/clusters",
            &json!({
                "name": format!("Cluster {}", &Uuid::new_v4().simple().to_string()[..6]),
                "address": "Testweg 1",
                "description": "Execution",
                "soil_condition": "Su3",
                "tree_ids": [tree_id],
                "organization_id": ORG,
            }),
            dispatcher,
        )
        .await;
    assert_eq!(cluster.status().as_u16(), 201, "cluster setup failed");
    let cluster_id = cluster.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let user_ids: Vec<String> = assignee.map(|u| u.to_string()).into_iter().collect();
    let plan = app
        .post_json_with_bearer(
            "/api/v1/plans",
            &json!({
                "date": date,
                "description": "Fahrer-Tour",
                "transporter_id": transporter["id"],
                "module": "irrigation",
                "stops": [cluster_id],
                "user_ids": user_ids,
                "organization_id": ORG,
            }),
            dispatcher,
        )
        .await;
    assert_eq!(plan.status().as_u16(), 201, "plan setup failed");
    let plan_id = plan.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    (plan_id, cluster_id)
}

const DATE: &str = "2026-08-15T06:00:00Z";

#[tokio::test]
async fn assigned_driver_with_read_right_runs_the_whole_tour() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (driver, driver_id) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (plan, cluster) = planned_plan(&app, &dispatcher, Some(driver_id), DATE).await;

    let start = app
        .post_json_with_bearer(&format!("/api/v1/plans/{plan}/start"), &json!({}), &driver)
        .await;
    assert_eq!(start.status().as_u16(), 200);
    let body: serde_json::Value = start.json().await.unwrap();
    assert_eq!(body["status"], "active");

    let outcomes = json!({ "outcomes": [{ "stop": cluster, "completed": true, "amount": 80.0 }] });
    let finish = app
        .post_json_with_bearer(&format!("/api/v1/plans/{plan}/finish"), &outcomes, &driver)
        .await;
    assert_eq!(finish.status().as_u16(), 200);
    assert_eq!(
        finish.json::<serde_json::Value>().await.unwrap()["status"],
        "finished"
    );
}

#[tokio::test]
async fn unassigned_reader_is_forbidden() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (other, _) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (plan, _) = planned_plan(&app, &dispatcher, None, DATE).await;

    let resp = app
        .post_json_with_bearer(&format!("/api/v1/plans/{plan}/start"), &json!({}), &other)
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[tokio::test]
async fn assigned_user_without_read_right_sees_404() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (blind, blind_id) = seed_user_in_org(&harness, &app, &["vehicle:read"]).await;
    let (plan, _) = planned_plan(&app, &dispatcher, Some(blind_id), DATE).await;

    let resp = app
        .post_json_with_bearer(&format!("/api/v1/plans/{plan}/start"), &json!({}), &blind)
        .await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn dispatcher_with_update_right_may_execute_unassigned() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (plan, _) = planned_plan(&app, &dispatcher, None, DATE).await;

    let resp = app
        .post_json_with_bearer(
            &format!("/api/v1/plans/{plan}/start"),
            &json!({}),
            &dispatcher,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn repeated_start_and_finish_are_idempotent_but_different_outcomes_conflict() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (driver, driver_id) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (plan, cluster) = planned_plan(&app, &dispatcher, Some(driver_id), DATE).await;
    let start = format!("/api/v1/plans/{plan}/start");
    let finish = format!("/api/v1/plans/{plan}/finish");
    let same = json!({ "outcomes": [{ "stop": cluster, "completed": true, "amount": 80.0 }] });
    let other = json!({ "outcomes": [{ "stop": cluster, "completed": true, "amount": 99.0 }] });

    for _ in 0..2 {
        let resp = app.post_json_with_bearer(&start, &json!({}), &driver).await;
        assert_eq!(resp.status().as_u16(), 200);
    }
    for _ in 0..2 {
        let resp = app.post_json_with_bearer(&finish, &same, &driver).await;
        assert_eq!(resp.status().as_u16(), 200);
    }
    let conflict = app.post_json_with_bearer(&finish, &other, &driver).await;
    assert_eq!(conflict.status().as_u16(), 409);
    assert_eq!(
        conflict.json::<serde_json::Value>().await.unwrap()["code"],
        "plan.already_finished_differently"
    );
}

#[tokio::test]
async fn fail_needs_a_note_and_finish_after_cancel_is_rejected() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (driver, driver_id) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (plan, cluster) = planned_plan(&app, &dispatcher, Some(driver_id), DATE).await;
    app.post_json_with_bearer(&format!("/api/v1/plans/{plan}/start"), &json!({}), &driver)
        .await;

    let blank = app
        .post_json_with_bearer(
            &format!("/api/v1/plans/{plan}/fail"),
            &json!({ "note": " " }),
            &driver,
        )
        .await;
    assert_eq!(blank.status().as_u16(), 400);

    let current: serde_json::Value = app
        .get_with_bearer(&format!("/api/v1/plans/{plan}"), &dispatcher)
        .await
        .json()
        .await
        .unwrap();
    let cancel = json!({
        "date": current["date"],
        "description": current["description"],
        "transporter_id": current["transporter"]["id"],
        "stops": current["stops"],
        "user_ids": current["user_ids"],
        "status": "canceled",
        "cancellation_note": "Disposition: Wetter",
    });
    let canceled = app
        .put_json_with_bearer(&format!("/api/v1/plans/{plan}"), &cancel, &dispatcher)
        .await;
    assert_eq!(canceled.status().as_u16(), 200);

    let finish = app
        .post_json_with_bearer(
            &format!("/api/v1/plans/{plan}/finish"),
            &json!({ "outcomes": [{ "stop": cluster, "completed": true, "amount": 80.0 }] }),
            &driver,
        )
        .await;
    assert_eq!(finish.status().as_u16(), 400);
    assert_eq!(
        finish.json::<serde_json::Value>().await.unwrap()["code"],
        "plan.invalid_state_transition"
    );
}
