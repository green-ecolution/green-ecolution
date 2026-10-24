use crate::helpers::spawn_app;

#[tokio::test]
async fn list_regions_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/regions").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_regions_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/regions").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_region_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/regions/{}", uuid::Uuid::now_v7()))
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_region_returns_200_for_existing_region() {
    let app = spawn_app().await;

    let region_id = uuid::Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO regions (id, name) VALUES ($1, 'Altstadt')",
        region_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.get(&format!("/api/v1/regions/{}", region_id)).await;

    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["name"], "Altstadt");
}

#[tokio::test]
async fn list_regions_returns_inserted_regions() {
    let app = spawn_app().await;

    let r1 = uuid::Uuid::now_v7();
    let r2 = uuid::Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO regions (id, name) VALUES ($1, 'Altstadt'), ($2, 'Neustadt')",
        r1,
        r2,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = app.get("/api/v1/regions").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 2);
}

#[tokio::test]
async fn list_regions_respects_pagination() {
    let app = spawn_app().await;

    for i in 1..=5 {
        sqlx::query!(
            "INSERT INTO regions (id, name) VALUES ($1, $2)",
            uuid::Uuid::now_v7(),
            format!("Region {}", i)
        )
        .execute(&app.db_pool)
        .await
        .unwrap();
    }

    let response = app.get("/api/v1/regions?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
    assert!(body["pagination"]["next_page"].as_u64().is_some());
}
