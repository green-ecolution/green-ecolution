use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

use crate::helpers::{TestApp, disabled_auth_settings, spawn_app_with_valhalla_and_auth};
use crate::routing::{
    create_named_cluster_with_tree, create_transporter, mock_streamlet, plan_body, streamlet_ok,
};

async fn routed_plan(valhalla: &MockServer) -> (TestApp, String, MockServer) {
    let streamlet = mock_streamlet(ResponseTemplate::new(200).set_body_json(streamlet_ok())).await;
    let app = spawn_app_with_valhalla_and_auth(
        &streamlet.uri(),
        &valhalla.uri(),
        disabled_auth_settings(),
    )
    .await;
    let transporter = create_transporter(&app).await;
    let cluster = create_named_cluster_with_tree(&app, "Nav").await;
    let plan: serde_json::Value = app
        .post_json(
            "/api/v1/plans",
            &plan_body(transporter["id"].as_str().unwrap(), &cluster),
        )
        .await
        .json()
        .await
        .unwrap();
    (app, plan["id"].as_str().unwrap().to_owned(), streamlet)
}

#[tokio::test]
async fn navigation_passes_the_valhalla_answer_through() {
    let valhalla = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .and(body_partial_json(
            json!({ "format": "osrm", "costing": "truck" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"code":"Ok","routes":[]}"#))
        .mount(&valhalla)
        .await;
    let (app, plan, _streamlet) = routed_plan(&valhalla).await;

    let resp = app
        .post_json(
            &format!("/api/v1/plans/{plan}/navigation"),
            &json!({ "lat": 54.79, "lon": 9.43, "heading": 90.0, "remaining": [1] }),
        )
        .await;

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().await.unwrap(), r#"{"code":"Ok","routes":[]}"#);
}

#[tokio::test]
async fn out_of_range_index_is_400() {
    let valhalla = MockServer::start().await;
    let (app, plan, _streamlet) = routed_plan(&valhalla).await;
    let resp = app
        .post_json(
            &format!("/api/v1/plans/{plan}/navigation"),
            &json!({ "lat": 54.79, "lon": 9.43, "remaining": [999] }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn a_second_call_within_two_seconds_is_429() {
    let valhalla = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&valhalla)
        .await;
    let (app, plan, _streamlet) = routed_plan(&valhalla).await;
    let body = json!({ "lat": 54.79, "lon": 9.43, "remaining": [1] });
    let url = format!("/api/v1/plans/{plan}/navigation");
    assert_eq!(app.post_json(&url, &body).await.status().as_u16(), 200);
    let second = app.post_json(&url, &body).await;
    assert_eq!(second.status().as_u16(), 429);
    assert_eq!(
        second.json::<serde_json::Value>().await.unwrap()["code"],
        "routing.too_many_requests"
    );
}

#[tokio::test]
async fn valhalla_down_is_502() {
    let valhalla = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/route"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&valhalla)
        .await;
    let (app, plan, _streamlet) = routed_plan(&valhalla).await;
    let resp = app
        .post_json(
            &format!("/api/v1/plans/{plan}/navigation"),
            &json!({ "lat": 54.79, "lon": 9.43, "remaining": [1] }),
        )
        .await;
    assert_eq!(resp.status().as_u16(), 502);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["code"],
        "routing.unavailable"
    );
}
