use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method, matchers::path};

use crate::auth_helpers::AuthHarness;
use crate::helpers::spawn_app_with_routing;
use crate::helpers::spawn_app_with_routing_and_auth;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";
const TBZ_ORG: &str = "01980000-0000-7000-8000-000000000002";

async fn mock_streamlet() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/solve"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    server
}

async fn insert_tbz_org(app: &crate::helpers::TestApp) {
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, '01980000-0000-7000-8000-000000000001', 'TBZ')",
        uuid::Uuid::parse_str(TBZ_ORG).unwrap()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

fn start_point_payload(name: &str, lat: f64, lon: f64, org: Option<&str>) -> serde_json::Value {
    let mut p = json!({ "name": name, "lat": lat, "lon": lon, "watering_point": false });
    if let Some(org) = org {
        p["organization_id"] = json!(org);
    }
    p
}

#[tokio::test]
async fn create_start_point_stores_explicit_organization() {
    let streamlet = mock_streamlet().await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    insert_tbz_org(&app).await;

    let resp = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot TBZ", 54.7, 9.4, Some(TBZ_ORG)),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["organization_id"], TBZ_ORG);
}

#[tokio::test]
async fn two_depots_in_different_orgs_can_both_be_default() {
    let streamlet = mock_streamlet().await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    insert_tbz_org(&app).await;

    let root_depot: serde_json::Value = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot Root", 54.6, 9.3, None),
        )
        .await
        .json()
        .await
        .unwrap();
    let tbz_depot: serde_json::Value = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot TBZ", 54.7, 9.4, Some(TBZ_ORG)),
        )
        .await
        .json()
        .await
        .unwrap();

    let root_resp = app
        .post_json(
            &format!(
                "/api/v1/routing/start-points/{}/default",
                root_depot["id"].as_str().unwrap()
            ),
            &json!({}),
        )
        .await;
    assert_eq!(root_resp.status(), 204);

    let tbz_resp = app
        .post_json(
            &format!(
                "/api/v1/routing/start-points/{}/default",
                tbz_depot["id"].as_str().unwrap()
            ),
            &json!({}),
        )
        .await;
    assert_eq!(tbz_resp.status(), 204);

    let defaults: Vec<(uuid::Uuid, bool)> =
        sqlx::query_as("SELECT organization_id, is_default FROM depots WHERE is_default")
            .fetch_all(&app.db_pool)
            .await
            .unwrap();
    let root_org_id = uuid::Uuid::parse_str(ROOT_ORG).unwrap();
    let tbz_org_id = uuid::Uuid::parse_str(TBZ_ORG).unwrap();
    assert!(
        defaults
            .iter()
            .any(|(org, is_default)| *org == root_org_id && *is_default),
        "root org must still have a default depot"
    );
    assert!(
        defaults
            .iter()
            .any(|(org, is_default)| *org == tbz_org_id && *is_default),
        "TBZ org must have its own default depot"
    );
}

#[tokio::test]
async fn set_default_in_one_org_leaves_other_orgs_default_untouched() {
    let streamlet = mock_streamlet().await;
    let app = spawn_app_with_routing(&streamlet.uri()).await;
    insert_tbz_org(&app).await;

    // Root org already has a seeded default ("Betriebshof Schleswiger Straße").
    let tbz_depot: serde_json::Value = app
        .post_json(
            "/api/v1/routing/start-points",
            &start_point_payload("Depot TBZ", 54.7, 9.4, Some(TBZ_ORG)),
        )
        .await
        .json()
        .await
        .unwrap();

    let resp = app
        .post_json(
            &format!(
                "/api/v1/routing/start-points/{}/default",
                tbz_depot["id"].as_str().unwrap()
            ),
            &json!({}),
        )
        .await;
    assert_eq!(resp.status(), 204);

    let root_default: String =
        sqlx::query_scalar("SELECT name FROM depots WHERE is_default AND organization_id = $1")
            .bind(uuid::Uuid::parse_str(ROOT_ORG).unwrap())
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(
        root_default, "Betriebshof Schleswiger Straße",
        "setting a default in TBZ must not touch the root org's default"
    );

    let tbz_default: bool = sqlx::query_scalar("SELECT is_default FROM depots WHERE id = $1")
        .bind(uuid::Uuid::parse_str(tbz_depot["id"].as_str().unwrap()).unwrap())
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert!(tbz_default);
}

#[tokio::test]
async fn create_start_point_requires_create_permission_in_target_org() {
    let streamlet = mock_streamlet().await;
    let harness = AuthHarness::start().await;
    let app = spawn_app_with_routing_and_auth(&streamlet.uri(), harness.auth_settings(true)).await;
    let token = harness.sign_token(json!({ "sub": uuid::Uuid::now_v7().to_string() }));
    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/routing/start-points", app.address))
        .bearer_auth(&token)
        .json(&start_point_payload("Depot X", 54.7, 9.4, Some(ROOT_ORG)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
