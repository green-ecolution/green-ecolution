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
