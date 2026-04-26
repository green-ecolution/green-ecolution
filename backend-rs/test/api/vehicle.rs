use crate::helpers::spawn_app;

fn vehicle_json(plate: &str) -> serde_json::Value {
    serde_json::json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2,
        "width": 2.5,
        "length": 8.0,
        "weight": 12000.0
    })
}

#[tokio::test]
async fn list_vehicles_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/vehicles").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_vehicles_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/vehicles").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"], 0);
}

#[tokio::test]
async fn get_vehicles_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/vehicles/999").await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_vehicle_returns_201() {
    let app = spawn_app().await;

    let response = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 123")).await;

    assert_eq!(response.status().as_u16(), 201);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["number_plate"], "FL-GE 123");
    assert_eq!(vehicle["model"], "MAN TGS");
    assert_eq!(vehicle["water_capacity"], 5000.0);
    assert_eq!(vehicle["status"], "available");
    assert_eq!(vehicle["type"], "transporter");
    assert_eq!(vehicle["driving_license"], "C");
}

#[tokio::test]
async fn create_vehicle_with_negative_capacity_returns_400() {
    let app = spawn_app().await;

    let mut body = vehicle_json("FL-GE 999");
    body["water_capacity"] = serde_json::json!(-100.0);

    let response = app.post_json("/api/v1/vehicles", &body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_vehicle_returns_full_response() {
    let app = spawn_app().await;

    let create_resp = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 100")).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let response = app.get(&format!("/api/v1/vehicles/{}", id)).await;

    assert_eq!(response.status().as_u16(), 200);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["number_plate"], "FL-GE 100");
    assert_eq!(vehicle["height"], 3.2);
    assert_eq!(vehicle["width"], 2.5);
    assert_eq!(vehicle["length"], 8.0);
    assert_eq!(vehicle["weight"], 12000.0);
}

#[tokio::test]
async fn get_vehicle_by_plate_returns_vehicle() {
    let app = spawn_app().await;

    app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 200")).await;

    let response = app.get("/api/v1/vehicles/plate/FL-GE 200").await;

    assert_eq!(response.status().as_u16(), 200);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["number_plate"], "FL-GE 200");
}

#[tokio::test]
async fn update_vehicle_changes_model() {
    let app = spawn_app().await;

    let create_resp = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 300")).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let mut update_body = vehicle_json("FL-GE 300");
    update_body["model"] = serde_json::json!("Mercedes Actros");

    let response = app
        .put_json(&format!("/api/v1/vehicles/{}", id), &update_body)
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["model"], "Mercedes Actros");
}

#[tokio::test]
async fn delete_vehicle_returns_204() {
    let app = spawn_app().await;

    let create_resp = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 400")).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let response = app.delete(&format!("/api/v1/vehicles/{}", id)).await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get(&format!("/api/v1/vehicles/{}", id)).await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn archive_vehicle_hides_from_default_list() {
    let app = spawn_app().await;

    let create_resp = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 500")).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let archive_resp = app
        .post_json(&format!("/api/v1/vehicles/archived/{}", id), &serde_json::json!({}))
        .await;
    assert_eq!(archive_resp.status().as_u16(), 204);

    let list_resp = app.get("/api/v1/vehicles").await;
    let list_body: serde_json::Value = list_resp.json().await.unwrap();
    assert_eq!(list_body["data"].as_array().unwrap().len(), 0);

    let archived_resp = app.get("/api/v1/vehicles/archived").await;
    let archived_body: serde_json::Value = archived_resp.json().await.unwrap();
    assert_eq!(archived_body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_vehicles_respects_pagination() {
    let app = spawn_app().await;

    for i in 1..=5 {
        app.post_json("/api/v1/vehicles", &vehicle_json(&format!("FL-GE {}", i)))
            .await;
    }

    let response = app.get("/api/v1/vehicles?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
}

#[tokio::test]
async fn create_duplicate_plate_returns_409() {
    let app = spawn_app().await;

    app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 999")).await;

    let response = app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 999")).await;

    assert_eq!(response.status().as_u16(), 409);
}
