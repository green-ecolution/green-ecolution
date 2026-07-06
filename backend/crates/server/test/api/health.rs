use crate::helpers::spawn_app;

#[tokio::test]
async fn get_health_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/health").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn get_health_lives_under_api_but_not_versioned() {
    let app = spawn_app().await;

    assert_eq!(app.get("/health").await.status().as_u16(), 404);
    assert_eq!(app.get("/api/v1/health").await.status().as_u16(), 404);
}

#[tokio::test]
async fn get_ready_returns_200_when_dependencies_healthy() {
    let app = spawn_app().await;

    let response = app.get("/api/ready").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("readiness body is json");
    assert_eq!(body["ready"], serde_json::Value::Bool(true));
}

#[tokio::test]
async fn get_ready_returns_503_when_database_unreachable() {
    let app = spawn_app().await;
    app.db_pool.close().await;

    let response = app.get("/api/ready").await;

    assert_eq!(response.status().as_u16(), 503);
    let body: serde_json::Value = response.json().await.expect("readiness body is json");
    assert_eq!(body["ready"], serde_json::Value::Bool(false));
}
