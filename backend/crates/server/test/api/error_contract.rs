//! Every failing response must be JSON with a stable `code`.
//!
//! The unit tests cover the extractors in isolation; these go through the real
//! router, which is the only place the fallbacks and the layer stack apply.

use crate::helpers::spawn_app;

fn content_type(response: &reqwest::Response) -> Option<&str> {
    response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn a_malformed_uuid_in_the_path_is_rejected_as_json() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees/not-a-uuid").await;

    assert_eq!(response.status(), 400);
    assert!(
        content_type(&response).is_some_and(|value| value.starts_with("application/json")),
        "got {:?}",
        content_type(&response)
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "request.malformed_path_parameter");
}

#[tokio::test]
async fn a_malformed_query_parameter_is_rejected_as_json() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees?page=not-a-number").await;

    assert_eq!(response.status(), 400);
    assert!(
        content_type(&response).is_some_and(|value| value.starts_with("application/json")),
        "got {:?}",
        content_type(&response)
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "request.malformed_query_parameter");
}

#[tokio::test]
async fn an_unknown_endpoint_answers_with_a_json_body() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/there-is-no-such-thing").await;

    assert_eq!(response.status(), 404);
    assert!(
        content_type(&response).is_some_and(|value| value.starts_with("application/json")),
        "an empty body reads as a parse failure, not as a 404: got {:?}",
        content_type(&response)
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "request.unknown_endpoint");
}

#[tokio::test]
async fn a_wrong_method_answers_with_a_json_body() {
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .delete(format!("{}/api/v1/trees", app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), 405);
    let content = content_type(&response).map(str::to_owned);
    assert!(
        content
            .as_deref()
            .is_some_and(|value| value.starts_with("application/json")),
        "got {content:?}"
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "request.method_not_allowed");
}
