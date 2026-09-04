use crate::helpers::spawn_app;

fn vehicle_json(plate: &str) -> serde_json::Value {
    serde_json::json!({
        "number_plate": plate,
        "description": "Testfahrzeug",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "availability": "available",
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
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_vehicles_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/vehicles/{}", uuid::Uuid::now_v7()))
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_vehicle_returns_201() {
    let app = spawn_app().await;

    let response = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 123"))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["number_plate"], "FL-GE 123");
    assert_eq!(vehicle["model"], "MAN TGS");
    assert_eq!(vehicle["water_capacity"], 5000.0);
    assert_eq!(vehicle["status"], "available");
    assert_eq!(vehicle["availability"], "available");
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

    let create_resp = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 100"))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

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

    app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 200"))
        .await;

    let response = app.get("/api/v1/vehicles/plate/FL-GE 200").await;

    assert_eq!(response.status().as_u16(), 200);

    let vehicle: serde_json::Value = response.json().await.unwrap();
    assert_eq!(vehicle["number_plate"], "FL-GE 200");
}

#[tokio::test]
async fn update_vehicle_changes_model() {
    let app = spawn_app().await;

    let create_resp = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 300"))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

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

    let create_resp = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 400"))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let response = app.delete(&format!("/api/v1/vehicles/{}", id)).await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get(&format!("/api/v1/vehicles/{}", id)).await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn archive_vehicle_hides_from_default_list() {
    let app = spawn_app().await;

    let create_resp = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 500"))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let archive_resp = app
        .post_json(
            &format!("/api/v1/vehicles/archived/{}", id),
            &serde_json::json!({}),
        )
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
    assert_eq!(body["pagination"]["total_records"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
}

#[tokio::test]
async fn create_duplicate_plate_returns_409() {
    let app = spawn_app().await;

    app.post_json("/api/v1/vehicles", &vehicle_json("FL-GE 999"))
        .await;

    let response = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 999"))
        .await;

    assert_eq!(response.status().as_u16(), 409);
}

#[tokio::test]
async fn update_vehicle_with_string_number_returns_json_error_body() {
    let app = spawn_app().await;

    let create_resp = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 400"))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let mut update_body = vehicle_json("FL-GE 400");
    update_body["height"] = serde_json::json!("1.88");

    let response = app
        .put_json(&format!("/api/v1/vehicles/{}", id), &update_body)
        .await;

    assert_eq!(response.status().as_u16(), 422);
    let body: serde_json::Value = response
        .json()
        .await
        .expect("malformed body must still yield a JSON error response");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("height"),
        "error body should name the offending field, got: {body}"
    );
}

#[tokio::test]
async fn vehicle_status_follows_the_plan_it_is_assigned_to() {
    let app = spawn_app().await;

    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 500"))
        .await
        .json()
        .await
        .unwrap();
    let vehicle_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "available");

    let plan: serde_json::Value = app
        .post_json(
            "/api/v1/watering-plans",
            &serde_json::json!({
                "date": "2026-05-01T08:00:00Z",
                "description": "Bewaesserung Innenstadt",
                "transporter_id": vehicle_id,
                "tree_cluster_ids": [],
                "user_ids": []
            }),
        )
        .await
        .json()
        .await
        .unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let planned: serde_json::Value = app
        .get(&format!("/api/v1/vehicles/{}", vehicle_id))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        planned["status"], "available",
        "a merely planned run must not make the vehicle active"
    );

    let plan_update = |status: &str| {
        serde_json::json!({
            "date": "2026-05-01T08:00:00Z",
            "description": "Bewaesserung Innenstadt",
            "status": status,
            "transporter_id": vehicle_id,
            "tree_cluster_ids": [],
            "user_ids": [],
            "cancellation_note": "",
            "evaluation": [],
        })
    };

    let start = app
        .put_json(
            &format!("/api/v1/watering-plans/{}", plan_id),
            &plan_update("active"),
        )
        .await;
    assert_eq!(start.status().as_u16(), 200);

    let active: serde_json::Value = app
        .get(&format!("/api/v1/vehicles/{}", vehicle_id))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(active["status"], "active");
    assert_eq!(
        active["availability"], "available",
        "the stored availability stays untouched while the plan runs"
    );

    let finish = app
        .put_json(
            &format!("/api/v1/watering-plans/{}", plan_id),
            &plan_update("finished"),
        )
        .await;
    assert_eq!(finish.status().as_u16(), 200);

    let after: serde_json::Value = app
        .get(&format!("/api/v1/vehicles/{}", vehicle_id))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(after["status"], "available");
}

#[tokio::test]
async fn not_available_vehicle_stays_not_available_on_an_active_plan() {
    let app = spawn_app().await;

    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &vehicle_json("FL-GE 600"))
        .await
        .json()
        .await
        .unwrap();
    let vehicle_id = created["id"].as_str().unwrap();

    let plan: serde_json::Value = app
        .post_json(
            "/api/v1/watering-plans",
            &serde_json::json!({
                "date": "2026-05-01T08:00:00Z",
                "description": "Bewaesserung Innenstadt",
                "transporter_id": vehicle_id,
                "tree_cluster_ids": [],
                "user_ids": []
            }),
        )
        .await
        .json()
        .await
        .unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let start = app
        .put_json(
            &format!("/api/v1/watering-plans/{}", plan_id),
            &serde_json::json!({
                "date": "2026-05-01T08:00:00Z",
                "description": "Bewaesserung Innenstadt",
                "status": "active",
                "transporter_id": vehicle_id,
                "tree_cluster_ids": [],
                "user_ids": [],
                "cancellation_note": "",
                "evaluation": [],
            }),
        )
        .await;
    assert_eq!(start.status().as_u16(), 200);

    let mut workshop = vehicle_json("FL-GE 600");
    workshop["availability"] = serde_json::json!("not_available");
    let update = app
        .put_json(&format!("/api/v1/vehicles/{}", vehicle_id), &workshop)
        .await;
    assert_eq!(update.status().as_u16(), 200);

    let updated: serde_json::Value = update.json().await.unwrap();
    assert_eq!(
        updated["status"], "not_available",
        "a vehicle in the workshop must not read as active"
    );
}
