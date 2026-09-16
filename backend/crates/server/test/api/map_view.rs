use serde_json::json;
use uuid::Uuid;

use crate::{
    auth_helpers::{AuthHarness, spawn_with_auth},
    helpers::{TestApp, seed_user_with_permissions, spawn_app},
    organizations::ROOT_ORG_ID,
};

/// Matches `config/base.yaml`, which the test settings are built from.
const DEFAULT_CENTER: [f64; 2] = [54.792277136221905, 9.43580607453268];
const DEFAULT_BBOX: [f64; 4] = [54.714822, 9.285796, 54.860127, 9.583800];
const DEFAULT_MIN_ZOOM: u8 = 13;
const DEFAULT_MAX_ZOOM: u8 = 18;

/// Somewhere else entirely, so a leaked default is never mistaken for a hit.
const HAMBURG_CENTER: [f64; 2] = [53.55, 9.99];
const HAMBURG_BBOX: [f64; 4] = [53.40, 9.75, 53.70, 10.25];

const MAP_VIEW_PATH: &str = "/api/v1/users/me/map-view";

fn root() -> Uuid {
    Uuid::parse_str(ROOT_ORG_ID).unwrap()
}

fn hamburg() -> serde_json::Value {
    json!({ "center": HAMBURG_CENTER, "bbox": HAMBURG_BBOX, "min_zoom": 11, "max_zoom": 19 })
}

/// A user inside an existing organization, holding exactly `permissions`. The
/// shared helper always opens a fresh organization, but these tests need two
/// accounts to meet in the same one.
async fn token_in_org(
    harness: &AuthHarness,
    app: &TestApp,
    org: Uuid,
    role_name: &str,
    permissions: &[&str],
) -> String {
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, $2, $3)
           RETURNING id"#,
        org,
        role_name,
        &permissions,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
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

async fn set_map_view(app: &TestApp, org: Uuid, token: &str, view: serde_json::Value) {
    let resp = app
        .put_json_with_bearer(
            &format!("/api/v1/organizations/{org}/settings"),
            &json!({ "map_view": view }),
            token,
        )
        .await;
    assert_eq!(
        resp.status(),
        200,
        "failed to store the organization's view"
    );
}

async fn map_view_of(app: &TestApp, token: &str) -> serde_json::Value {
    let resp = app.get_with_bearer(MAP_VIEW_PATH, token).await;
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

#[tokio::test]
async fn an_account_without_an_organization_gets_the_instance_default() {
    let app = spawn_app().await;

    let resp = app.get(MAP_VIEW_PATH).await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["center"], json!(DEFAULT_CENTER));
    assert_eq!(body["bbox"], json!(DEFAULT_BBOX));
    assert_eq!(body["min_zoom"], DEFAULT_MIN_ZOOM);
    assert_eq!(body["max_zoom"], DEFAULT_MAX_ZOOM);
}

#[tokio::test]
async fn a_signed_in_user_gets_their_own_organizations_view() {
    let (harness, app) = spawn_with_auth().await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Hamburg", &["setting:update"]).await;
    set_map_view(&app, org, &writer, hamburg()).await;
    // Deliberately without any permission: the opening viewport is the
    // caller's own, not a reading of someone else's configuration.
    let member = token_in_org(&harness, &app, org, "Ohne Rechte", &[]).await;

    let body = map_view_of(&app, &member).await;

    assert_eq!(body["center"], json!(HAMBURG_CENTER));
    assert_eq!(body["bbox"], json!(HAMBURG_BBOX));
    assert_eq!(body["min_zoom"], 11);
    assert_eq!(body["max_zoom"], 19);
}

#[tokio::test]
async fn an_organization_without_its_own_view_inherits_down_the_tree() {
    let (harness, app) = spawn_with_auth().await;
    let root_writer =
        token_in_org(&harness, &app, root(), "Instanz-Admin", &["setting:update"]).await;
    set_map_view(&app, root(), &root_writer, hamburg()).await;
    let (_org, leaf_member) = seed_user_with_permissions(&harness, &app, "Blatt", &[]).await;

    let body = map_view_of(&app, &leaf_member).await;

    assert_eq!(body["center"], json!(HAMBURG_CENTER));
    assert_eq!(body["bbox"], json!(HAMBURG_BBOX));
}

#[tokio::test]
async fn an_own_value_beats_the_one_inherited_from_above() {
    let (harness, app) = spawn_with_auth().await;
    let root_writer =
        token_in_org(&harness, &app, root(), "Instanz-Admin", &["setting:update"]).await;
    set_map_view(&app, root(), &root_writer, hamburg()).await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Eigenwillig", &["setting:update"]).await;
    set_map_view(
        &app,
        org,
        &writer,
        json!({
            "center": DEFAULT_CENTER, "bbox": DEFAULT_BBOX,
            "min_zoom": DEFAULT_MIN_ZOOM, "max_zoom": DEFAULT_MAX_ZOOM
        }),
    )
    .await;

    let body = map_view_of(&app, &writer).await;

    assert_eq!(body["center"], json!(DEFAULT_CENTER));
}

#[tokio::test]
async fn the_viewport_is_not_readable_without_a_token() {
    let (_harness, app) = spawn_with_auth().await;

    let resp = app.get(MAP_VIEW_PATH).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "an organization's whereabouts must not be readable before the login"
    );
}

/// The reason `/info/map` may stay public at all: it must never become an
/// answer to "which tenants exist and where do they sit", which is exactly
/// what it would be if an organization's own value reached it.
#[tokio::test]
async fn info_map_keeps_reporting_the_instance_default() {
    let (harness, app) = spawn_with_auth().await;
    let root_writer =
        token_in_org(&harness, &app, root(), "Instanz-Admin", &["setting:update"]).await;
    set_map_view(&app, root(), &root_writer, hamburg()).await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Hamburg", &["setting:update"]).await;
    set_map_view(&app, org, &writer, hamburg()).await;

    let resp = app.get("/api/v1/info/map").await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["center"], json!(DEFAULT_CENTER));
    assert_eq!(body["bbox"], json!(DEFAULT_BBOX));
    assert_eq!(body["min_zoom"], DEFAULT_MIN_ZOOM);
    assert_eq!(body["max_zoom"], DEFAULT_MAX_ZOOM);
}

#[tokio::test]
async fn a_viewport_without_a_box_leaves_the_map_unrestricted() {
    let (harness, app) = spawn_with_auth().await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Ohne Grenze", &["setting:update"]).await;

    set_map_view(
        &app,
        org,
        &writer,
        json!({ "center": HAMBURG_CENTER, "bbox": null }),
    )
    .await;

    let body = map_view_of(&app, &writer).await;
    assert_eq!(body["center"], json!(HAMBURG_CENTER));
    assert!(
        body["bbox"].is_null(),
        "an unrestricted viewport must not invent a box, got {body}"
    );
    // The zoom range goes with the box: lifting the limit lifts all of it.
    assert!(body["min_zoom"].is_null(), "got {body}");
    assert!(body["max_zoom"].is_null(), "got {body}");
}

#[tokio::test]
async fn a_zoom_range_without_a_box_is_refused() {
    let app = spawn_app().await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{}/settings", root()),
            &json!({ "map_view": { "center": HAMBURG_CENTER, "min_zoom": 11, "max_zoom": 19 } }),
        )
        .await;

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn an_inverted_zoom_range_is_refused() {
    let app = spawn_app().await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{}/settings", root()),
            &json!({ "map_view": {
                "center": HAMBURG_CENTER, "bbox": HAMBURG_BBOX,
                "min_zoom": 19, "max_zoom": 11
            } }),
        )
        .await;

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn a_zoom_level_beyond_the_scale_is_refused() {
    let app = spawn_app().await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{}/settings", root()),
            &json!({ "map_view": {
                "center": HAMBURG_CENTER, "bbox": HAMBURG_BBOX,
                "min_zoom": 11, "max_zoom": 25
            } }),
        )
        .await;

    assert_eq!(resp.status(), 400);
}

/// The generated client parses a null bbox back into `undefined` and then
/// omits the key, so a viewport that came from this very API must still be
/// accepted on the way back in.
#[tokio::test]
async fn an_omitted_box_reads_the_same_as_an_explicit_null() {
    let (harness, app) = spawn_with_auth().await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Ohne Feld", &["setting:update"]).await;

    set_map_view(&app, org, &writer, json!({ "center": HAMBURG_CENTER })).await;

    let body = map_view_of(&app, &writer).await;
    assert_eq!(body["center"], json!(HAMBURG_CENTER));
    assert!(body["bbox"].is_null());
}

/// The centre rule has nothing to hold against without a box, so a centre that
/// a bounded viewport would reject has to pass here.
#[tokio::test]
async fn a_centre_anywhere_is_allowed_without_a_box() {
    let (harness, app) = spawn_with_auth().await;
    let (org, writer) =
        seed_user_with_permissions(&harness, &app, "Weit weg", &["setting:update"]).await;

    set_map_view(
        &app,
        org,
        &writer,
        json!({ "center": [48.13, 11.58], "bbox": null }),
    )
    .await;

    let body = map_view_of(&app, &writer).await;
    assert_eq!(body["center"], json!([48.13, 11.58]));
}

/// Dropping the limit is a value like any other and must reach the subtree.
#[tokio::test]
async fn an_unrestricted_viewport_is_inherited_downwards() {
    let (harness, app) = spawn_with_auth().await;
    let root_writer =
        token_in_org(&harness, &app, root(), "Instanz-Admin", &["setting:update"]).await;
    set_map_view(
        &app,
        root(),
        &root_writer,
        json!({ "center": HAMBURG_CENTER, "bbox": null }),
    )
    .await;
    let (_org, leaf_member) = seed_user_with_permissions(&harness, &app, "Blatt", &[]).await;

    let body = map_view_of(&app, &leaf_member).await;
    assert_eq!(body["center"], json!(HAMBURG_CENTER));
    assert!(body["bbox"].is_null());
}

/// The instance default always carries a box, so the public endpoint keeps
/// answering with one even after an organization drops its own.
#[tokio::test]
async fn info_map_still_carries_a_box_when_an_organization_drops_its_own() {
    let (harness, app) = spawn_with_auth().await;
    let root_writer =
        token_in_org(&harness, &app, root(), "Instanz-Admin", &["setting:update"]).await;
    set_map_view(
        &app,
        root(),
        &root_writer,
        json!({ "center": HAMBURG_CENTER, "bbox": null }),
    )
    .await;

    let resp = app.get("/api/v1/info/map").await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["bbox"], json!(DEFAULT_BBOX));
}

#[tokio::test]
async fn a_centre_outside_the_box_is_refused() {
    let app = spawn_app().await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{}/settings", root()),
            &json!({ "map_view": { "center": [48.13, 11.58], "bbox": HAMBURG_BBOX } }),
        )
        .await;

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap().contains("map_view"),
        "the message must name the offending field, got {body}"
    );
}

#[tokio::test]
async fn an_inverted_box_is_refused() {
    let app = spawn_app().await;

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{}/settings", root()),
            // North below south, east below west.
            &json!({ "map_view": { "center": HAMBURG_CENTER, "bbox": [53.70, 10.25, 53.40, 9.75] } }),
        )
        .await;

    assert_eq!(resp.status(), 400);
}
