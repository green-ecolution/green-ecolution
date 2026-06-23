use crate::helpers::spawn_app;

#[tokio::test]
async fn response_carries_generated_x_request_id_header() {
    let app = spawn_app().await;

    let response = app.get("/health").await;

    let header = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok());

    assert!(
        header.is_some_and(|id| !id.is_empty()),
        "every response must carry a non-empty x-request-id header, got {header:?}"
    );
}

#[tokio::test]
async fn response_echoes_client_supplied_x_request_id_header() {
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .get(format!("{}/health", app.address))
        .header("x-request-id", "test-correlation-123")
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok()),
        Some("test-correlation-123"),
    );
}
