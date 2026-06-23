use crate::helpers::spawn_app;

#[tokio::test]
async fn get_health_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/health").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn get_health_has_no_api_prefix() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/health").await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_ready_returns_200_when_dependencies_healthy() {
    let app = spawn_app().await;

    let response = app.get("/ready").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("readiness body is json");
    assert_eq!(body["ready"], serde_json::Value::Bool(true));
}

#[tokio::test]
async fn get_ready_returns_503_when_database_unreachable() {
    let app = spawn_app().await;
    app.db_pool.close().await;

    let response = app.get("/ready").await;

    assert_eq!(response.status().as_u16(), 503);
    let body: serde_json::Value = response.json().await.expect("readiness body is json");
    assert_eq!(body["ready"], serde_json::Value::Bool(false));
}
