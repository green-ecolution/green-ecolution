use std::sync::Arc;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use server::infra::update_checker::UpdateChecker;

#[tokio::test]
async fn no_update_when_versions_match() {
    let checker = UpdateChecker::new("0.1.0".into(), None);
    assert_eq!(checker.latest().await, "0.1.0");
    assert!(!checker.update_available().await);
}

#[tokio::test]
async fn refresh_no_op_when_repo_unset() {
    let checker = UpdateChecker::new("0.1.0".into(), None);
    let client = reqwest::Client::new();
    checker.refresh_with_base(&client, "http://unused/").await.unwrap();
    assert_eq!(checker.latest().await, "0.1.0");
}

#[tokio::test]
async fn refresh_reports_newer_release() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/org/repo/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v0.2.0"
        })))
        .mount(&mock)
        .await;

    let checker = Arc::new(UpdateChecker::new("0.1.0".into(), Some("org/repo".into())));
    let client = reqwest::Client::new();
    checker.refresh_with_base(&client, &mock.uri()).await.unwrap();

    assert_eq!(checker.latest().await, "0.2.0");
    assert!(checker.update_available().await);
}

#[tokio::test]
async fn refresh_strips_leading_v_for_comparison() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/org/repo/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v0.1.0"
        })))
        .mount(&mock)
        .await;

    let checker = Arc::new(UpdateChecker::new("0.1.0".into(), Some("org/repo".into())));
    let client = reqwest::Client::new();
    checker.refresh_with_base(&client, &mock.uri()).await.unwrap();

    assert!(!checker.update_available().await);
}

#[tokio::test]
async fn refresh_survives_500() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/org/repo/releases/latest"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;

    let checker = UpdateChecker::new("0.1.0".into(), Some("org/repo".into()));
    let client = reqwest::Client::new();
    let result = checker.refresh_with_base(&client, &mock.uri()).await;
    assert!(result.is_err());
    assert_eq!(checker.latest().await, "0.1.0");
    assert!(!checker.update_available().await);
}

#[tokio::test]
async fn refresh_survives_malformed_body() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/org/repo/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock)
        .await;

    let checker = UpdateChecker::new("0.1.0".into(), Some("org/repo".into()));
    let client = reqwest::Client::new();
    let result = checker.refresh_with_base(&client, &mock.uri()).await;
    assert!(result.is_err());
    assert_eq!(checker.latest().await, "0.1.0");
}
