use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app},
    module_host::switch,
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
};

const KEY: &str = "waste";

fn root() -> Uuid {
    ROOT_ORG_ID.parse().unwrap()
}

pub async fn waste_app() -> TestApp {
    let app = spawn_app().await;
    assert_eq!(switch(&app, KEY, root(), Some(true)).await.status(), 200);
    app
}

pub fn bin_body(number: &str) -> Value {
    json!({
        "number": number,
        "latitude": 54.79,
        "longitude": 9.44,
        "volumeLiters": 1100.0,
        "depthCm": 120
    })
}

pub async fn create_bin(app: &TestApp, number: &str) -> Value {
    let res = app.post_json("/api/v1/waste-bins", &bin_body(number)).await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    res.json().await.unwrap()
}

pub fn bin_target(id: impl std::fmt::Display) -> Value {
    json!({ "target": { "module": KEY, "kind": "bin", "id": id.to_string() } })
}

fn id_of(bin: &Value) -> Uuid {
    bin["id"].as_str().unwrap().parse().unwrap()
}

async fn insert_bin(app: &TestApp, number: &str, org: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO waste_bins (id, number, latitude, longitude, geometry, volume_liters, depth_cm, organization_id)
         VALUES ($1, $2, 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 1100, 120, $3)",
    )
    .bind(id)
    .bind(number)
    .bind(org)
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

async fn set_fill(
    app: &TestApp,
    id: Uuid,
    fraction: Option<f64>,
    measured_hours_ago: Option<i64>,
    emptied_hours_ago: Option<i64>,
) {
    sqlx::query(
        "UPDATE waste_bins SET
            fill_fraction = $2,
            fill_measured_at = now() - make_interval(hours => $3::int),
            last_emptied_at = now() - make_interval(hours => $4::int)
         WHERE id = $1",
    )
    .bind(id)
    .bind(fraction)
    .bind(measured_hours_ago.map(|h| h as i32))
    .bind(emptied_hours_ago.map(|h| h as i32))
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn list(app: &TestApp, query: &str) -> Value {
    let res = app.get(&format!("/api/v1/waste-bins{query}")).await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());
    res.json().await.unwrap()
}

fn numbers(list: &Value) -> Vec<String> {
    list["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["number"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn a_bin_can_be_created_read_and_listed() {
    let app = waste_app().await;
    let created = create_bin(&app, "AB-1").await;
    assert_eq!(created["fillState"], "unknown");
    assert_eq!(created["volumeLiters"], 1100.0);
    assert_eq!(created["depthCm"], 120);
    assert_eq!(created["organizationId"], ROOT_ORG_ID);
    assert!(created["createdAt"].is_string());
    assert!(created.get("fillPercent").is_none());
    let got: Value = app
        .get(&format!(
            "/api/v1/waste-bins/{}",
            created["id"].as_str().unwrap()
        ))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(got["number"], "AB-1");
    let list: Value = app.get("/api/v1/waste-bins").await.json().await.unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_bin_number_is_unique_within_an_organization() {
    let app = waste_app().await;
    create_bin(&app, "AB-1").await;
    let res = app.post_json("/api/v1/waste-bins", &bin_body("AB-1")).await;
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "waste_bin.number_taken"
    );
}

#[tokio::test]
async fn renaming_onto_a_taken_number_is_a_conflict() {
    let app = waste_app().await;
    create_bin(&app, "AB-1").await;
    let second = create_bin(&app, "AB-2").await;
    let res = app
        .put_json(
            &format!("/api/v1/waste-bins/{}", id_of(&second)),
            &bin_body("AB-1"),
        )
        .await;
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "waste_bin.number_taken"
    );
}

#[tokio::test]
async fn a_bin_can_be_updated() {
    let app = waste_app().await;
    let bin = create_bin(&app, "AB-1").await;
    let res = app
        .put_json(
            &format!("/api/v1/waste-bins/{}", id_of(&bin)),
            &json!({
                "number": "AB-9",
                "latitude": 54.8,
                "longitude": 9.45,
                "volumeLiters": 240.0,
                "depthCm": 90
            }),
        )
        .await;
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["number"], "AB-9");
    assert_eq!(body["volumeLiters"], 240.0);
    assert_eq!(body["depthCm"], 90);
    assert_eq!(body["latitude"], 54.8);
}

#[tokio::test]
async fn writes_are_refused_where_the_module_is_off() {
    let app = spawn_app().await;
    let res = app.post_json("/api/v1/waste-bins", &bin_body("AB-1")).await;
    assert_eq!(res.status(), 403);
    assert_eq!(
        res.json::<Value>().await.unwrap()["code"],
        "module.disabled"
    );
}

#[tokio::test]
async fn the_list_hides_bins_where_the_module_is_off() {
    let app = waste_app().await;
    let child = child_org(&app, "Ohne Abfall", root()).await;
    assert_eq!(switch(&app, KEY, child, Some(false)).await.status(), 200);
    create_bin(&app, "AB-1").await;
    let hidden = insert_bin(&app, "HIDDEN-1", child).await;

    assert_eq!(numbers(&list(&app, "").await), ["AB-1"]);
    assert_eq!(
        app.get(&format!("/api/v1/waste-bins/{hidden}"))
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn an_invalid_depth_is_rejected() {
    let app = waste_app().await;
    let mut body = bin_body("AB-1");
    body["depthCm"] = json!(0);
    let res = app.post_json("/api/v1/waste-bins", &body).await;
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "request.validation_failed");
    assert_eq!(body["validation"]["field"], "waste_bin.depth");
}

#[tokio::test]
async fn the_list_filters_by_fill_state() {
    let app = waste_app().await;
    let full = id_of(&create_bin(&app, "FULL").await);
    let normal = id_of(&create_bin(&app, "NORMAL").await);
    let empty = id_of(&create_bin(&app, "EMPTY").await);
    create_bin(&app, "NEVER").await;
    let stale = id_of(&create_bin(&app, "STALE").await);
    set_fill(&app, full, Some(0.9), Some(1), None).await;
    set_fill(&app, normal, Some(0.3), Some(1), Some(5)).await;
    set_fill(&app, empty, Some(0.9), Some(2), Some(1)).await;
    set_fill(&app, stale, Some(0.9), Some(24 * 60), None).await;

    let full_list = list(&app, "?fillState=full").await;
    assert_eq!(numbers(&full_list), ["FULL"]);
    assert_eq!(full_list["data"][0]["fillState"], "full");
    assert_eq!(full_list["data"][0]["fillPercent"], 90);
    assert_eq!(numbers(&list(&app, "?fillState=normal").await), ["NORMAL"]);
    assert_eq!(numbers(&list(&app, "?fillState=empty").await), ["EMPTY"]);
    assert_eq!(
        numbers(&list(&app, "?fillState=unknown").await),
        ["NEVER", "STALE"]
    );
    assert_eq!(
        numbers(
            &list(
                &app,
                &format!("?fillState=full&organizationId={ROOT_ORG_ID}")
            )
            .await
        ),
        ["FULL"]
    );
}

#[tokio::test]
async fn the_list_searches_by_number_and_pages() {
    let app = waste_app().await;
    for n in ["AB-1", "AB-2", "XY-1"] {
        create_bin(&app, n).await;
    }
    assert_eq!(numbers(&list(&app, "?number=ab").await), ["AB-1", "AB-2"]);
    let page = list(&app, "?page=2&perPage=2").await;
    assert_eq!(numbers(&page), ["XY-1"]);
    assert_eq!(page["pagination"]["total_records"], 3);
}

#[tokio::test]
async fn a_bin_without_sensor_moves_to_another_organization() {
    let app = waste_app().await;
    let child = child_org(&app, "Bauhof", root()).await;
    let bin = create_bin(&app, "AB-1").await;
    let res = app
        .patch_json(
            &format!("/api/v1/waste-bins/{}/organization", id_of(&bin)),
            &json!({ "organizationId": child }),
        )
        .await;
    assert_eq!(res.status(), 204);
    let got: Value = app
        .get(&format!("/api/v1/waste-bins/{}", id_of(&bin)))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(got["organizationId"], child.to_string());
}

#[tokio::test]
async fn a_bin_can_be_deleted() {
    let app = waste_app().await;
    let bin = create_bin(&app, "AB-1").await;
    let url = format!("/api/v1/waste-bins/{}", id_of(&bin));
    assert_eq!(app.delete(&url).await.status(), 204);
    assert_eq!(app.get(&url).await.status(), 404);
}

#[tokio::test]
#[ignore = "needs task 6/7"]
async fn deleting_a_bin_releases_its_sensor_removes_it_from_plans_and_drops_comments() {
    let app = waste_app().await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let other = id_of(&create_bin(&app, "AB-2").await);

    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-bin-1").await;
    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &bin_target(bin),
        )
        .await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());

    let res = app
        .post_json(
            &format!("/api/v1/targets/waste/bin/{bin}/comments"),
            &json!({ "body": "Deckel klemmt" }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());

    let vehicle =
        crate::vehicle::create_vehicle_with(&app, json!({ "cargo_volume": 6000.0 })).await;
    let res = app
        .post_json(
            "/api/v1/plans",
            &json!({
                "module": KEY,
                "date": "2026-11-02T08:00:00Z",
                "description": "Leerung",
                "transporter_id": vehicle,
                "stops": [bin, other],
                "user_ids": [],
                "transporter_capacity_used": true,
                "organization_id": ROOT_ORG_ID
            }),
        )
        .await;
    assert_eq!(res.status(), 201, "{}", res.text().await.unwrap());
    let plan: Value = res.json().await.unwrap();

    assert_eq!(
        app.delete(&format!("/api/v1/waste-bins/{bin}"))
            .await
            .status(),
        204
    );

    let sensor: Value = app
        .get(&format!("/api/v1/sensors/{sensor}"))
        .await
        .json()
        .await
        .unwrap();
    assert!(sensor["target"].is_null());

    let plan: Value = app
        .get(&format!("/api/v1/plans/{}", plan["id"].as_str().unwrap()))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(plan["stops"], json!([other]));

    let comments: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comments
         WHERE subject_type = 'target' AND target_kind = 'bin' AND subject_id = $1",
    )
    .bind(bin)
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(comments, 0);
}

#[tokio::test]
#[ignore = "needs task 6/7"]
async fn transferring_a_bin_moves_its_sensor_along() {
    let app = waste_app().await;
    let child = child_org(&app, "Bauhof", root()).await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let sensor = crate::sensors::create_prepared_sensor(&app, "eui-bin-2").await;
    let res = app
        .post_json(
            &format!("/api/v1/sensors/{sensor}/activate"),
            &bin_target(bin),
        )
        .await;
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());

    let res = app
        .patch_json(
            &format!("/api/v1/waste-bins/{bin}/organization"),
            &json!({ "organizationId": child }),
        )
        .await;
    assert_eq!(res.status(), 204);

    let got: Value = app
        .get(&format!("/api/v1/waste-bins/{bin}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(got["organizationId"], child.to_string());
    assert_eq!(got["sensorId"], sensor);

    let sensor: Value = app
        .get(&format!("/api/v1/sensors/{sensor}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(sensor["organization_id"], child.to_string());
    assert_eq!(sensor["target"]["id"], bin.to_string());
}
