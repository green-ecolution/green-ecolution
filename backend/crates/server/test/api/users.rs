use crate::helpers::spawn_app;

fn valid_body() -> serde_json::Value {
    serde_json::json!({
        "employee_id": "EMP-1",
        "phone_number": "+49 461 1",
        "avatar_url": "",
        "status": "absent",
        "driving_licenses": ["B", "CE"]
    })
}

#[tokio::test]
async fn update_user_returns_200_for_demo_user() {
    let app = spawn_app().await;

    let response = app
        .put_json(
            "/api/v1/users/00000000-0000-0000-0000-000000000000",
            &valid_body(),
        )
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["username"], "ttester");
}

#[tokio::test]
async fn update_user_returns_404_for_unknown_user() {
    let app = spawn_app().await;

    let response = app
        .put_json(
            &format!("/api/v1/users/{}", uuid::Uuid::now_v7()),
            &valid_body(),
        )
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn update_user_returns_400_for_invalid_avatar_url() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "avatar_url": "not a url",
        "status": "absent",
        "driving_licenses": []
    });
    let response = app
        .put_json("/api/v1/users/00000000-0000-0000-0000-000000000000", &body)
        .await;

    assert_eq!(response.status().as_u16(), 400);
}
