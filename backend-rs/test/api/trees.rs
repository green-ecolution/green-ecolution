use crate::helpers::spawn_app;

#[tokio::test]
async fn list_trees_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_trees_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"], 0);
}

#[tokio::test]
async fn get_trees_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees/999").await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_tree_returns_201() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-001",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Testbaum"
    });

    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 201);

    let tree: serde_json::Value = response.json().await.unwrap();
    assert_eq!(tree["species"], "Eiche");
    assert_eq!(tree["number"], "T-001");
    assert_eq!(tree["planting_year"], 2020);
    assert_eq!(tree["watering_status"], "unknown");
}

#[tokio::test]
async fn create_tree_with_invalid_coordinates_returns_400() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-001",
        "planting_year": 2020,
        "latitude": 999.0,
        "longitude": 9.99,
        "description": "Testbaum"
    });

    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_tree_with_future_planting_year_returns_400() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-001",
        "planting_year": 3000,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Testbaum"
    });

    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_tree_with_cluster_links_it() {
    let app = spawn_app().await;

    let cluster_body = serde_json::json!({
        "name": "Stadtpark",
        "address": "Parkweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": []
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_i64().unwrap();

    let tree_body = serde_json::json!({
        "species": "Buche",
        "number": "T-002",
        "planting_year": 2018,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Baum im Cluster",
        "tree_cluster_id": cluster_id
    });

    let response = app.post_json("/api/v1/trees", &tree_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let tree: serde_json::Value = response.json().await.unwrap();
    assert_eq!(tree["tree_cluster_id"], cluster_id);
}

#[tokio::test]
async fn get_tree_returns_full_response() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Linde",
        "number": "T-010",
        "planting_year": 2015,
        "latitude": 53.56,
        "longitude": 10.01,
        "description": "Alte Linde"
    });

    let create_resp = app.post_json("/api/v1/trees", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let response = app.get(&format!("/api/v1/trees/{}", id)).await;

    assert_eq!(response.status().as_u16(), 200);

    let tree: serde_json::Value = response.json().await.unwrap();
    assert_eq!(tree["species"], "Linde");
    assert_eq!(tree["number"], "T-010");
    assert_eq!(tree["latitude"], 53.56);
    assert_eq!(tree["longitude"], 10.01);
}

#[tokio::test]
async fn update_tree_changes_species() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-001",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Alt"
    });

    let create_resp = app.post_json("/api/v1/trees", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let update_body = serde_json::json!({
        "species": "Buche",
        "number": "T-001",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Aktualisiert"
    });

    let response = app
        .put_json(&format!("/api/v1/trees/{}", id), &update_body)
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let tree: serde_json::Value = response.json().await.unwrap();
    assert_eq!(tree["species"], "Buche");
    assert_eq!(tree["description"], "Aktualisiert");
}

#[tokio::test]
async fn delete_tree_returns_204() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Birke",
        "number": "T-DEL",
        "planting_year": 2019,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Wird geloescht"
    });

    let create_resp = app.post_json("/api/v1/trees", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let response = app.delete(&format!("/api/v1/trees/{}", id)).await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get(&format!("/api/v1/trees/{}", id)).await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn list_trees_respects_pagination() {
    let app = spawn_app().await;

    for i in 1..=5 {
        let body = serde_json::json!({
            "species": format!("Baum {}", i),
            "number": format!("T-{:03}", i),
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "Test"
        });
        app.post_json("/api/v1/trees", &body).await;
    }

    let response = app.get("/api/v1/trees?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
}

#[tokio::test]
async fn list_planting_years_returns_distinct_years() {
    let app = spawn_app().await;

    for (i, year) in [2018, 2020, 2020, 2022].iter().enumerate() {
        let body = serde_json::json!({
            "species": "Eiche",
            "number": format!("T-{:03}", i + 1),
            "planting_year": year,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "Test"
        });
        app.post_json("/api/v1/trees", &body).await;
    }

    let response = app.get("/api/v1/trees/planting-years").await;

    assert_eq!(response.status().as_u16(), 200);

    let years: Vec<i32> = response.json().await.unwrap();
    assert_eq!(years, vec![2018, 2020, 2022]);
}
