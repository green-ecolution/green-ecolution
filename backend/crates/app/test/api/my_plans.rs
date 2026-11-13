use serde_json::json;

use crate::auth_helpers::spawn_with_auth;
use crate::plan_execution::{DISPATCH, DRIVER, planned_plan, seed_user_in_org};

#[tokio::test]
async fn lists_only_plans_assigned_to_the_caller_within_the_window() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (driver, driver_id) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (mine, _) = planned_plan(&app, &dispatcher, Some(driver_id), "2026-08-15T06:00:00Z").await;
    let (_not_mine, _) = planned_plan(&app, &dispatcher, None, "2026-08-15T06:00:00Z").await;
    let (_too_late, _) =
        planned_plan(&app, &dispatcher, Some(driver_id), "2026-09-30T06:00:00Z").await;

    let resp = app
        .get_with_bearer(
            "/api/v1/users/me/plans?from=2026-08-14&to=2026-08-20",
            &driver,
        )
        .await;

    assert_eq!(resp.status().as_u16(), 200);
    let ids: Vec<String> = resp
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(ids, vec![mine]);
}

#[tokio::test]
async fn finished_plans_are_hidden_by_default() {
    let (harness, app) = spawn_with_auth().await;
    let (dispatcher, _) = seed_user_in_org(&harness, &app, DISPATCH).await;
    let (driver, driver_id) = seed_user_in_org(&harness, &app, DRIVER).await;
    let (plan, cluster) =
        planned_plan(&app, &dispatcher, Some(driver_id), "2026-08-15T06:00:00Z").await;
    app.post_json_with_bearer(&format!("/api/v1/plans/{plan}/start"), &json!({}), &driver)
        .await;
    let finished = app
        .post_json_with_bearer(
            &format!("/api/v1/plans/{plan}/finish"),
            &json!({ "outcomes": [{ "stop": cluster, "completed": true, "amount": 80.0 }] }),
            &driver,
        )
        .await;
    assert_eq!(finished.status().as_u16(), 200);

    let default: Vec<serde_json::Value> = app
        .get_with_bearer(
            "/api/v1/users/me/plans?from=2026-08-14&to=2026-08-20",
            &driver,
        )
        .await
        .json()
        .await
        .unwrap();
    assert!(default.is_empty());

    let with_finished: Vec<serde_json::Value> = app
        .get_with_bearer(
            "/api/v1/users/me/plans?from=2026-08-14&to=2026-08-20&status=finished",
            &driver,
        )
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(with_finished.len(), 1);
    assert_eq!(with_finished[0]["id"], plan.as_str());
}

#[tokio::test]
async fn a_window_over_31_days_is_400() {
    let (harness, app) = spawn_with_auth().await;
    let (driver, _) = seed_user_in_org(&harness, &app, DRIVER).await;
    let resp = app
        .get_with_bearer(
            "/api/v1/users/me/plans?from=2026-01-01&to=2026-03-01",
            &driver,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}
