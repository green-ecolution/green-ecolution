use crate::helpers::spawn_app;

#[tokio::test]
async fn get_evaluation_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/evaluation").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn get_evaluation_returns_zero_counts_on_empty_db() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/evaluation").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["tree_count"], 0);
    assert_eq!(body["treecluster_count"], 0);
    assert_eq!(body["sensor_count"], 0);
    assert_eq!(body["watering_plan_count"], 0);
    assert_eq!(body["total_water_consumption"], 0);
    assert!(body["region_evaluation"].as_array().unwrap().is_empty());
    assert!(body["vehicle_evaluation"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_evaluation_counts_trees_and_clusters() {
    let app = spawn_app().await;

    // Create trees
    for i in 1..=3 {
        let body = serde_json::json!({
            "species": "Eiche",
            "number": format!("T-EVAL-{}", i),
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "Test"
        });
        app.post_json("/api/v1/trees", &body).await;
    }

    // Create clusters
    for i in 1..=2 {
        let body = serde_json::json!({
            "name": format!("Cluster {}", i),
            "address": "Test",
            "description": "Test",
            "soil_condition": "sandig",
            "tree_ids": []
        });
        app.post_json("/api/v1/clusters", &body).await;
    }

    let response = app.get("/api/v1/evaluation").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["tree_count"], 3);
    assert_eq!(body["treecluster_count"], 2);
}

#[tokio::test]
async fn get_evaluation_counts_vehicles_and_sensors() {
    let app = spawn_app().await;

    // Create vehicles
    for i in 1..=2 {
        let body = serde_json::json!({
            "number_plate": format!("FL-GE {}", i),
            "description": "Test",
            "water_capacity": 5000.0,
            "model": "MAN",
            "status": "available",
            "type": "transporter",
            "driving_license": "C",
            "height": 3.0, "width": 2.5, "length": 8.0, "weight": 12000.0
        });
        app.post_json("/api/v1/vehicles", &body).await;
    }

    // Create sensors via SQL (no create endpoint)
    for i in 1..=4 {
        sqlx::query!(
            r#"INSERT INTO sensors (id, status, latitude, longitude, geometry)
            VALUES ($1, 'online', 53.55, 9.99, ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326))"#,
            format!("sensor-eval-{}", i)
        )
        .execute(&app.db_pool)
        .await
        .unwrap();
    }

    let response = app.get("/api/v1/evaluation").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["sensor_count"], 4);
    // Vehicles are in the total count (not in vehicle_evaluation which is about watering plans)
}

#[tokio::test]
async fn get_evaluation_includes_vehicle_watering_plan_stats() {
    let app = spawn_app().await;

    // Create a vehicle
    let vehicle_resp = app
        .post_json(
            "/api/v1/vehicles",
            &serde_json::json!({
                "number_plate": "FL-GE 100",
                "description": "Test",
                "water_capacity": 5000.0,
                "model": "MAN",
                "status": "available",
                "type": "transporter",
                "driving_license": "C",
                "height": 3.0, "width": 2.5, "length": 8.0, "weight": 12000.0
            }),
        )
        .await;
    let vehicle: serde_json::Value = vehicle_resp.json().await.unwrap();
    let vid = vehicle["id"].as_i64().unwrap();

    // Create watering plans with this vehicle
    for _ in 0..3 {
        app.post_json(
            "/api/v1/watering-plans",
            &serde_json::json!({
                "date": "2026-05-01T08:00:00Z",
                "description": "Test",
                "transporter_id": vid,
                "tree_cluster_ids": [],
                "user_ids": []
            }),
        )
        .await;
    }

    let response = app.get("/api/v1/evaluation").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["watering_plan_count"], 3);

    let vehicle_eval = body["vehicle_evaluation"].as_array().unwrap();
    assert_eq!(vehicle_eval.len(), 1);
    assert_eq!(vehicle_eval[0]["number_plate"], "FL-GE 100");
    assert_eq!(vehicle_eval[0]["watering_plan_count"], 3);
}

#[tokio::test]
async fn get_evaluation_includes_region_watering_plan_stats() {
    let app = spawn_app().await;

    // Create region with geometry
    let wkt = "MULTIPOLYGON(((9.98 53.54, 10.0 53.54, 10.0 53.56, 9.98 53.56, 9.98 53.54)))";
    sqlx::query!(
        r#"INSERT INTO regions (name, geometry) VALUES ($1, ST_SetSRID(ST_GeomFromText($2), 4326))"#,
        "Altstadt",
        wkt,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    // Create tree in region
    let tree_body = serde_json::json!({
        "species": "Eiche",
        "number": "T-REVAL",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Test"
    });
    app.post_json("/api/v1/trees", &tree_body).await;

    let tree_id: i32 = sqlx::query_scalar!("SELECT id FROM trees WHERE number = 'T-REVAL'")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();

    // Create cluster with tree (triggers region assignment)
    let cluster_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Eval Cluster",
                "address": "Test",
                "description": "Test",
                "soil_condition": "sandig",
                "tree_ids": [tree_id]
            }),
        )
        .await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cid = cluster["id"].as_i64().unwrap();

    // Create vehicle + watering plan linked to cluster
    let vehicle_resp = app
        .post_json(
            "/api/v1/vehicles",
            &serde_json::json!({
                "number_plate": "FL-GE 200",
                "description": "Test",
                "water_capacity": 5000.0,
                "model": "MAN",
                "status": "available",
                "type": "transporter",
                "driving_license": "C",
                "height": 3.0, "width": 2.5, "length": 8.0, "weight": 12000.0
            }),
        )
        .await;
    let vehicle: serde_json::Value = vehicle_resp.json().await.unwrap();
    let vid = vehicle["id"].as_i64().unwrap();

    app.post_json(
        "/api/v1/watering-plans",
        &serde_json::json!({
            "date": "2026-06-01T08:00:00Z",
            "description": "Test",
            "transporter_id": vid,
            "tree_cluster_ids": [cid],
            "user_ids": []
        }),
    )
    .await;

    let response = app.get("/api/v1/evaluation").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let region_eval = body["region_evaluation"].as_array().unwrap();
    assert_eq!(region_eval.len(), 1);
    assert_eq!(region_eval[0]["name"], "Altstadt");
    assert_eq!(region_eval[0]["watering_plan_count"], 1);
}
