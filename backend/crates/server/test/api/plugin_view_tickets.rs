use serde_json::json;

use crate::helpers::{TestApp, install_plugin, spawn_app_with_plugins};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

async fn install_with_view(app: &TestApp, slug: &str) -> String {
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/plugins",
            &json!({
                "slug": slug,
                "name": slug,
                "organization_id": ROOT_ORG,
                "permissions": [],
                "required_permissions": [],
                "frontend": { "mode": "external", "target": "https://plugin.example.org/view" }
            }),
        )
        .await
        .json()
        .await
        .unwrap();
    app.patch_json(
        &format!("/api/v1/plugins/{slug}"),
        &json!({ "enabled": true }),
    )
    .await;
    created["key"].as_str().unwrap().to_string()
}

async fn view_ticket(app: &TestApp, slug: &str) -> String {
    let view: serde_json::Value = app
        .get(&format!("/api/v1/plugins/{slug}/view"))
        .await
        .json()
        .await
        .unwrap();
    view["view_ticket"]
        .as_str()
        .expect("view must carry a ticket")
        .to_string()
}

async fn redeem(app: &TestApp, key: &str, ticket: &str) -> reqwest::Response {
    app.post_json_with_bearer(
        "/api/v1/plugins/view-tickets/redeem",
        &json!({ "ticket": ticket }),
        key,
    )
    .await
}

#[tokio::test]
async fn ticket_from_the_view_redeems_once() {
    let app = spawn_app_with_plugins().await;
    let key = install_with_view(&app, "sensor-setup").await;
    let ticket = view_ticket(&app, "sensor-setup").await;
    assert!(ticket.starts_with("gev_"));

    let first = redeem(&app, &key, &ticket).await;
    assert_eq!(first.status().as_u16(), 200);
    let body: serde_json::Value = first.json().await.unwrap();
    assert_eq!(body["organization_id"], ROOT_ORG);
    assert!(body["user"]["id"].is_string());
    assert!(body["user"]["display_name"].is_string());

    let second = redeem(&app, &key, &ticket).await;
    assert_eq!(second.status().as_u16(), 401);
    let body: serde_json::Value = second.json().await.unwrap();
    assert_eq!(body["code"], "plugin.view_ticket_invalid");
}

#[tokio::test]
async fn expired_ticket_is_rejected() {
    let app = spawn_app_with_plugins().await;
    let key = install_with_view(&app, "sensor-setup").await;
    let ticket = view_ticket(&app, "sensor-setup").await;
    sqlx::query("UPDATE plugin_view_tickets SET expires_at = now() - interval '1 second'")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let resp = redeem(&app, &key, &ticket).await;
    assert_eq!(resp.status().as_u16(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "plugin.view_ticket_invalid");
}

#[tokio::test]
async fn ticket_cannot_be_redeemed_by_another_plugin() {
    let app = spawn_app_with_plugins().await;
    install_with_view(&app, "sensor-setup").await;
    let other_key = install_plugin(&app, "other", &[]).await;
    let ticket = view_ticket(&app, "sensor-setup").await;

    let resp = redeem(&app, &other_key, &ticket).await;
    assert_eq!(resp.status().as_u16(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "plugin.view_ticket_invalid");
}

#[tokio::test]
async fn concurrent_redemption_succeeds_exactly_once() {
    let app = spawn_app_with_plugins().await;
    let key = install_with_view(&app, "sensor-setup").await;
    let ticket = view_ticket(&app, "sensor-setup").await;

    let (a, b) = tokio::join!(redeem(&app, &key, &ticket), redeem(&app, &key, &ticket));
    let mut statuses = [a.status().as_u16(), b.status().as_u16()];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 401]);
}

#[tokio::test]
async fn issuing_a_ticket_purges_expired_ones() {
    let app = spawn_app_with_plugins().await;
    install_with_view(&app, "sensor-setup").await;
    view_ticket(&app, "sensor-setup").await;
    sqlx::query("UPDATE plugin_view_tickets SET expires_at = now() - interval '1 second'")
        .execute(&app.db_pool)
        .await
        .unwrap();

    view_ticket(&app, "sensor-setup").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM plugin_view_tickets")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
