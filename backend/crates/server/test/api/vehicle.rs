use crate::helpers::{TestApp, spawn_app};

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

/// Creates a vehicle with the given plate/model/description, returning its id.
async fn seed_vehicle(app: &TestApp, plate: &str, model: &str, description: &str) -> uuid::Uuid {
    let mut body = vehicle_json(plate);
    body["model"] = serde_json::json!(model);
    body["description"] = serde_json::json!(description);
    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &body)
        .await
        .json()
        .await
        .unwrap();
    uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

async fn seed_vehicle_of_type(app: &TestApp, plate: &str, vehicle_type: &str) -> uuid::Uuid {
    let mut body = vehicle_json(plate);
    body["type"] = serde_json::json!(vehicle_type);
    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &body)
        .await
        .json()
        .await
        .unwrap();
    uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

async fn seed_vehicle_with_license(
    app: &TestApp,
    plate: &str,
    driving_license: &str,
) -> uuid::Uuid {
    let mut body = vehicle_json(plate);
    body["driving_license"] = serde_json::json!(driving_license);
    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &body)
        .await
        .json()
        .await
        .unwrap();
    uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

async fn seed_vehicle_with_capacity(app: &TestApp, plate: &str, capacity: f64) -> uuid::Uuid {
    let mut body = vehicle_json(plate);
    body["water_capacity"] = serde_json::json!(capacity);
    let created: serde_json::Value = app
        .post_json("/api/v1/vehicles", &body)
        .await
        .json()
        .await
        .unwrap();
    uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

/// Creates a watering plan for `vehicle_id` and starts it, so the vehicle's
/// derived status reads `active`. Mirrors the plan-lifecycle calls in
/// `vehicle_status_follows_the_plan_it_is_assigned_to`.
async fn put_vehicle_on_active_plan(app: &TestApp, vehicle_id: uuid::Uuid) {
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
}

// Raw sqlx::query so this fixture doesn't need an offline-cache entry.
async fn set_unavailable(app: &TestApp, id: uuid::Uuid) {
    sqlx::query("UPDATE vehicles SET availability = 'not_available' WHERE id = $1")
        .bind(id)
        .execute(&app.db_pool)
        .await
        .expect("test fixture updates vehicle availability");
}

#[tokio::test]
async fn vehicle_list_searches_plate_model_and_description() {
    let app = spawn_app().await;
    seed_vehicle(&app, "FL-GE 100", "MAN TGE", "Innenstadt").await;
    seed_vehicle(&app, "SL-XY 200", "Iveco Daily", "Hafen").await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?q=iveco")
        .await
        .json()
        .await
        .unwrap();

    let plates: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["number_plate"].as_str().unwrap())
        .collect();
    assert_eq!(plates, vec!["SL-XY 200"]);
}

#[tokio::test]
async fn vehicle_list_filters_by_derived_status() {
    let app = spawn_app().await;
    let available = seed_vehicle(&app, "FL-GE 100", "MAN TGE", "").await;
    let blocked = seed_vehicle(&app, "FL-GE 200", "MAN TGE", "").await;
    set_unavailable(&app, blocked).await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?status=not_available")
        .await
        .json()
        .await
        .unwrap();

    let ids: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![blocked.to_string()]);
    assert!(!ids.contains(&available.to_string().as_str()));
}

#[tokio::test]
async fn vehicle_list_status_filter_agrees_with_the_rust_derivation() {
    // The SQL CASE and vehicle::derive_status answer the same question in two
    // places; this pins them together so one cannot drift. `active` is the
    // interesting branch: SQL evaluates an EXISTS over vehicle_watering_plans
    // joined to watering_plans, while Rust receives a pre-computed bool, so a
    // fixture without a genuinely active vehicle would let this pass
    // vacuously.
    let app = spawn_app().await;
    seed_vehicle(&app, "FL-GE 100", "MAN TGE", "").await;
    let blocked = seed_vehicle(&app, "FL-GE 200", "MAN TGE", "").await;
    set_unavailable(&app, blocked).await;
    let active_vehicle = seed_vehicle(&app, "FL-GE 300", "MAN TGE", "").await;
    put_vehicle_on_active_plan(&app, active_vehicle).await;

    let all: serde_json::Value = app
        .get("/api/v1/vehicles?per_page=100")
        .await
        .json()
        .await
        .unwrap();

    for status in ["available", "not_available", "active"] {
        let expected: Vec<String> = all["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["status"].as_str() == Some(status))
            .map(|v| v["id"].as_str().unwrap().to_owned())
            .collect();

        if status == "active" {
            assert!(
                !expected.is_empty(),
                "fixture must contain a genuinely active vehicle, or this iteration \
                 would pass vacuously"
            );
        }

        let filtered: serde_json::Value = app
            .get(&format!("/api/v1/vehicles?per_page=100&status={status}"))
            .await
            .json()
            .await
            .unwrap();
        let actual: Vec<String> = filtered["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["id"].as_str().unwrap().to_owned())
            .collect();

        assert_eq!(actual, expected, "status filter disagrees for {status}");
    }
}

#[tokio::test]
async fn vehicle_list_type_parameter_is_repeatable() {
    let app = spawn_app().await;
    seed_vehicle_of_type(&app, "FL-GE 100", "transporter").await;
    seed_vehicle_of_type(&app, "FL-GE 200", "trailer").await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?type=transporter&type=trailer")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn vehicle_list_filters_by_driving_license() {
    let app = spawn_app().await;
    let ce = seed_vehicle_with_license(&app, "FL-GE 100", "CE").await;
    let b = seed_vehicle_with_license(&app, "FL-GE 200", "B").await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?driving_license=CE")
        .await
        .json()
        .await
        .unwrap();

    let ids: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![ce.to_string()]);
    assert!(!ids.contains(&b.to_string().as_str()));
}

#[tokio::test]
async fn vehicle_list_sorts_by_water_capacity_descending() {
    let app = spawn_app().await;
    seed_vehicle_with_capacity(&app, "FL-GE 100", 2000.0).await;
    seed_vehicle_with_capacity(&app, "FL-GE 200", 8000.0).await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?sort=water_capacity&order=desc")
        .await
        .json()
        .await
        .unwrap();

    let plates: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["number_plate"].as_str().unwrap())
        .collect();
    // Fixtures deliberately disagree with the default plate order, so a sort
    // that silently did nothing would fail here.
    assert_eq!(plates, vec!["FL-GE 200", "FL-GE 100"]);
}

#[tokio::test]
async fn vehicle_list_reports_the_prefilter_total() {
    let app = spawn_app().await;
    seed_vehicle(&app, "FL-GE 100", "MAN TGE", "").await;
    seed_vehicle(&app, "SL-XY 200", "Iveco Daily", "").await;

    let body: serde_json::Value = app
        .get("/api/v1/vehicles?q=iveco")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["pagination"]["total_records"], 1);
    assert_eq!(body["pagination"]["total_unfiltered"], 2);
}

#[tokio::test]
async fn vehicle_list_can_include_and_isolate_archived_vehicles() {
    let app = spawn_app().await;
    let active = seed_vehicle(&app, "FL-GE 100", "MAN TGE", "").await;
    let archived = seed_vehicle(&app, "FL-GE 200", "MAN TGE", "").await;
    app.post_json(
        &format!("/api/v1/vehicles/archived/{archived}"),
        &serde_json::json!({}),
    )
    .await;

    let default_list: serde_json::Value = app.get("/api/v1/vehicles").await.json().await.unwrap();
    assert_eq!(default_list["data"].as_array().unwrap().len(), 1);

    let including: serde_json::Value = app
        .get("/api/v1/vehicles?archive=include")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(including["data"].as_array().unwrap().len(), 2);

    let only: serde_json::Value = app
        .get("/api/v1/vehicles?archive=only")
        .await
        .json()
        .await
        .unwrap();
    let ids: Vec<&str> = only["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![archived.to_string()]);
    assert!(!ids.contains(&active.to_string().as_str()));
}
