use crate::helpers::{TestApp, spawn_app};

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
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_trees_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/trees/{}", uuid::Uuid::now_v7()))
        .await;

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
async fn create_tree_with_out_of_range_planting_year_returns_400() {
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
        "soil_condition": "Su3",
        "tree_ids": []
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

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
    let id = created["id"].as_str().unwrap();

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
    let id = created["id"].as_str().unwrap();

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
    let id = created["id"].as_str().unwrap();

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
    assert_eq!(body["pagination"]["total_records"], 5);
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

#[tokio::test]
async fn list_tree_markers_requires_bbox() {
    let app = spawn_app().await;
    let response = app.get("/api/v1/trees/markers").await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn list_tree_markers_returns_empty_for_far_bbox() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-001",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x"
    });
    let resp = app.post_json("/api/v1/trees", &body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app
        .get("/api/v1/trees/markers?bbox=52.4,13.3,52.6,13.5")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_tree_markers_returns_trees_inside_bbox() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-002",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x"
    });
    let resp = app.post_json("/api/v1/trees", &body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-002");
    assert_eq!(data[0]["has_sensor"], false);
    assert!(data[0]["latitude"].as_f64().unwrap() > 54.78);
}

#[tokio::test]
async fn list_tree_markers_combines_bbox_and_planting_year() {
    let app = spawn_app().await;

    let make = |number: &str, year: i32| {
        serde_json::json!({
            "species": "Eiche",
            "number": number,
            "planting_year": year,
            "latitude": 54.79,
            "longitude": 9.44,
            "description": "x"
        })
    };

    app.post_json("/api/v1/trees", &make("T-A", 2020)).await;
    app.post_json("/api/v1/trees", &make("T-B", 2021)).await;

    let resp = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&planting_year=2020")
        .await;
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-A");
}

#[tokio::test]
async fn list_tree_markers_rejects_malformed_bbox() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/trees/markers?bbox=garbage").await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn list_tree_markers_filters_by_watering_status() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-WS",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x"
    });
    let resp = app.post_json("/api/v1/trees", &body).await;
    assert_eq!(resp.status().as_u16(), 201);

    // The seeded tree's status is `unknown` (no sensor). Filtering by `unknown`
    // should return it; filtering by `good` should not.
    let resp = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&watering_status=unknown")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 1);

    let resp = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&watering_status=good")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_nearest_trees_returns_sorted_by_distance() {
    let app = spawn_app().await;

    // Three trees: T-NEAR (closest), T-MID, T-FAR.
    let make = |number: &str, lat: f64, lng: f64| {
        serde_json::json!({
            "species": "Eiche",
            "number": number,
            "planting_year": 2020,
            "latitude": lat,
            "longitude": lng,
            "description": "x"
        })
    };
    // ~111m/0.001° latitude — keep all three within the 500m default radius.
    app.post_json("/api/v1/trees", &make("T-NEAR", 54.7900, 9.4400))
        .await;
    app.post_json("/api/v1/trees", &make("T-MID", 54.7905, 9.4400))
        .await;
    app.post_json("/api/v1/trees", &make("T-FAR", 54.7920, 9.4400))
        .await;

    let resp = app
        .get("/api/v1/trees/nearest?lat=54.7900&lng=9.4400&limit=3")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 3);
    assert_eq!(data[0]["tree"]["number"], "T-NEAR");
    assert_eq!(data[1]["tree"]["number"], "T-MID");
    assert_eq!(data[2]["tree"]["number"], "T-FAR");

    let d0 = data[0]["distance_meters"].as_f64().unwrap();
    let d1 = data[1]["distance_meters"].as_f64().unwrap();
    let d2 = data[2]["distance_meters"].as_f64().unwrap();
    assert!(d0 <= d1 && d1 <= d2, "distances must be ascending");
    assert!(d0 < 1.0, "closest tree should be ~0m away, got {d0}");
}

#[tokio::test]
async fn get_nearest_trees_clamps_limit_to_max() {
    let app = spawn_app().await;

    // Seed more trees than max_limit (50) would be expensive; we instead verify
    // that an absurdly high limit is accepted (no 400) and the response contains
    // at most the existing trees.
    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-CLAMP",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x"
    });
    app.post_json("/api/v1/trees", &body).await;

    let resp = app
        .get("/api/v1/trees/nearest?lat=54.79&lng=9.44&limit=999999")
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(!json["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_nearest_trees_uses_default_limit_when_omitted() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-DEFLIM",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x"
    });
    app.post_json("/api/v1/trees", &body).await;

    let resp = app.get("/api/v1/trees/nearest?lat=54.79&lng=9.44").await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn get_nearest_trees_rejects_invalid_coordinates() {
    let app = spawn_app().await;

    let resp = app.get("/api/v1/trees/nearest?lat=200&lng=9.44").await;
    assert_eq!(resp.status().as_u16(), 400);

    let resp = app.get("/api/v1/trees/nearest?lat=54.79&lng=999").await;
    assert_eq!(resp.status().as_u16(), 400);
}

async fn create_tree_with(
    app: &TestApp,
    number: &str,
    planting_year: i32,
    cluster_id: Option<&str>,
) -> String {
    let mut body = serde_json::json!({
        "species": "Eiche",
        "number": number,
        "planting_year": planting_year,
        "latitude": 54.79,
        "longitude": 9.43,
        "description": "Testbaum"
    });
    if let Some(cid) = cluster_id {
        body["tree_cluster_id"] = serde_json::json!(cid);
    }
    let resp = app.post_json("/api/v1/trees", &body).await;
    assert_eq!(resp.status().as_u16(), 201);
    let tree: serde_json::Value = resp.json().await.unwrap();
    tree["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn list_tree_markers_accepts_repeated_filter_params() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;
    create_tree_with(&app, "T-002", 2020, None).await;
    create_tree_with(&app, "T-003", 2022, None).await;

    let response = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&planting_year=2018&planting_year=2020")
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_tree_markers_accepts_repeated_watering_statuses() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;

    let response = app
        .get("/api/v1/trees/markers?bbox=54.78,9.40,54.81,9.46&watering_status=good&watering_status=unknown")
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    // freshly created trees have status "unknown"
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn get_nearest_trees_excludes_trees_outside_radius() {
    let app = spawn_app().await;

    // T-CLOSE is within ~10m; T-OUT is ~2km away (outside the 500m default radius).
    let make = |number: &str, lat: f64, lng: f64| {
        serde_json::json!({
            "species": "Eiche",
            "number": number,
            "planting_year": 2020,
            "latitude": lat,
            "longitude": lng,
            "description": "x"
        })
    };
    app.post_json("/api/v1/trees", &make("T-CLOSE", 54.7900, 9.4400))
        .await;
    app.post_json("/api/v1/trees", &make("T-OUT", 54.8100, 9.4400))
        .await;

    let resp = app
        .get("/api/v1/trees/nearest?lat=54.7900&lng=9.4400&limit=10")
        .await;
    let json: serde_json::Value = resp.json().await.unwrap();
    let numbers: Vec<&str> = json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tree"]["number"].as_str().unwrap())
        .collect();
    assert!(numbers.contains(&"T-CLOSE"));
    assert!(!numbers.contains(&"T-OUT"));
}

async fn insert_q_seed(app: &TestApp) {
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description, organization_id)
        VALUES
            ($1, 2020, 'Eiche',  'T-001',    54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'a', '01980000-0000-7000-8000-000000000001'),
            ($2, 2021, 'Buche',  'T-50%STR', 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'b', '01980000-0000-7000-8000-000000000001'),
            ($3, 2022, 'Ahorn',  'T-1000',   54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'c', '01980000-0000-7000-8000-000000000001')"#,
        uuid::Uuid::now_v7(),
        uuid::Uuid::now_v7(),
        uuid::Uuid::now_v7(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn list_trees_filters_by_q_on_tree_number() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    let r = app.get("/api/v1/trees?q=T-001").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-001");
}

#[tokio::test]
async fn list_trees_filters_by_q_on_species() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    let r = app.get("/api/v1/trees?q=Buche").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["species"], "Buche");
}

#[tokio::test]
async fn list_trees_q_is_case_insensitive() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    let r = app.get("/api/v1/trees?q=eiche").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_trees_q_is_trimmed_and_treats_empty_as_unset() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    let r = app.get("/api/v1/trees?q=%20%20%20").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);

    let r = app.get("/api/v1/trees?q=%20Eiche%20").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_trees_q_escapes_like_wildcards() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    // `%25` is URL-encoded `%`; the handler must treat it as a literal `%`
    // in the LIKE pattern, matching only `T-50%STR` and not all three rows.
    let r = app.get("/api/v1/trees?q=50%25").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-50%STR");
}

#[tokio::test]
async fn list_trees_q_rejects_overlong_input() {
    let app = spawn_app().await;
    let long = "x".repeat(101);
    let r = app.get(&format!("/api/v1/trees?q={long}")).await;
    assert_eq!(r.status().as_u16(), 400);
}

#[tokio::test]
async fn list_trees_q_paginates_correctly() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    let r = app.get("/api/v1/trees?q=T-&per_page=2&page=1").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(body["pagination"]["total_records"], 3);
}

#[tokio::test]
async fn list_trees_q_treats_sql_injection_payload_as_literal() {
    let app = spawn_app().await;
    insert_q_seed(&app).await;

    for payload in &["' OR 1=1 --", "'; DROP TABLE trees;--", "%' OR '1"] {
        let url = format!("/api/v1/trees?q={}", urlencoding::encode(payload));
        let r = app.get(&url).await;
        assert_eq!(r.status().as_u16(), 200, "payload {payload} did not 200");
        let body: serde_json::Value = r.json().await.unwrap();
        assert_eq!(
            body["data"].as_array().unwrap().len(),
            0,
            "payload {payload} matched rows",
        );
    }

    // Confirm the table is still intact after injection attempts.
    let r = app.get("/api/v1/trees?q=Eiche").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_trees_filters_by_planting_year() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;
    create_tree_with(&app, "T-002", 2020, None).await;

    let response = app.get("/api/v1/trees?planting_year=2018").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-001");
    assert_eq!(body["pagination"]["total_records"], 1);
}

// Rows predating the lower bound exist (imports, the old unbounded API), and the
// year slider offers them. Filtering is a lookup and must not validate like a write.
#[tokio::test]
async fn list_trees_filter_accepts_planting_year_below_lower_bound() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;

    let response = app.get("/api/v1/trees?planting_year=25").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_trees_filters_by_multiple_planting_years() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;
    create_tree_with(&app, "T-002", 2020, None).await;
    create_tree_with(&app, "T-003", 2022, None).await;

    let response = app
        .get("/api/v1/trees?planting_year=2018&planting_year=2020")
        .await;

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_trees_filters_by_has_cluster() {
    let app = spawn_app().await;
    let cluster_body = serde_json::json!({
        "name": "Stadtpark",
        "address": "Parkweg 1",
        "description": "Test",
        "soil_condition": "Su3",
        "tree_ids": []
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    create_tree_with(&app, "T-001", 2018, Some(cluster_id)).await;
    create_tree_with(&app, "T-002", 2020, None).await;

    let response = app.get("/api/v1/trees?has_cluster=true").await;
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-001");

    let response = app.get("/api/v1/trees?has_cluster=false").await;
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-002");
}

#[tokio::test]
async fn list_trees_filters_by_watering_status() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;
    create_tree_with(&app, "T-002", 2020, None).await;

    // freshly created trees have status "unknown"
    let response = app.get("/api/v1/trees?watering_status=unknown").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    let response = app.get("/api/v1/trees?watering_status=good").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_trees_combines_filters() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;
    create_tree_with(&app, "T-002", 2020, None).await;

    let response = app
        .get("/api/v1/trees?planting_year=2018&watering_status=unknown")
        .await;

    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-001");
}

#[tokio::test]
async fn list_trees_rejects_invalid_watering_status() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees?watering_status=bogus").await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn list_trees_still_accepts_the_previous_just_watered_spelling() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;

    // The label was renamed to `just_watered`; the space-carrying spelling stays
    // accepted as a serde alias so existing callers keep working.
    let response = app
        .get("/api/v1/trees?watering_status=just%20watered")
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_trees_accepts_the_canonical_just_watered_status() {
    let app = spawn_app().await;
    create_tree_with(&app, "T-001", 2018, None).await;

    let response = app.get("/api/v1/trees?watering_status=just_watered").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// -- Sensor uniqueness: one sensor must never be linked to two trees --

async fn insert_sensor(app: &TestApp, id: &str) {
    let model_id = app.ecodrizzler_model_id().await;
    sqlx::query!(
        r#"INSERT INTO sensors (id, activated_at, type, model_id, organization_id)
        VALUES ($1, NOW(), 'lorawan', $2, '01980000-0000-7000-8000-000000000001')"#,
        id,
        model_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
        VALUES ($1, '', '', '', '')"#,
        id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

async fn insert_tree_with_sensor(app: &TestApp, number: &str, sensor_id: &str) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude,
                              geometry, description, sensor_id, watering_status, organization_id)
        VALUES ($1, 2020, 'Eiche', $2, 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', $3,
                $4::text::watering_status, '01980000-0000-7000-8000-000000000001')"#,
        id,
        number,
        Some(sensor_id),
        "unknown",
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn create_tree_with_sensor_of_other_tree_returns_409() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-conflict-1").await;
    insert_tree_with_sensor(&app, "T-CONF-1", "eui-conflict-1").await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-CONF-2",
        "planting_year": 2020,
        "latitude": 53.56,
        "longitude": 9.98,
        "description": "Testbaum",
        "sensor_id": "eui-conflict-1"
    });
    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 409);
}

#[tokio::test]
async fn update_tree_with_sensor_of_other_tree_returns_409() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-conflict-2").await;
    let holder = insert_tree_with_sensor(&app, "T-CONF-3", "eui-conflict-2").await;

    let create = serde_json::json!({
        "species": "Eiche",
        "number": "T-CONF-4",
        "planting_year": 2020,
        "latitude": 53.56,
        "longitude": 9.98,
        "description": "Testbaum"
    });
    let created: serde_json::Value = app
        .post_json("/api/v1/trees", &create)
        .await
        .json()
        .await
        .unwrap();
    let other_id = created["id"].as_str().unwrap();

    let update = serde_json::json!({
        "species": "Eiche",
        "number": "T-CONF-4",
        "planting_year": 2020,
        "latitude": 53.56,
        "longitude": 9.98,
        "description": "Testbaum",
        "sensor_id": "eui-conflict-2"
    });
    let response = app
        .put_json(&format!("/api/v1/trees/{other_id}"), &update)
        .await;

    assert_eq!(response.status().as_u16(), 409);

    // The sensor must still belong to the original tree
    let holder_after = app.get(&format!("/api/v1/trees/{holder}")).await;
    let holder_body: serde_json::Value = holder_after.json().await.unwrap();
    assert_eq!(holder_body["sensor"]["id"], "eui-conflict-2");
}

#[tokio::test]
async fn update_tree_keeping_its_own_sensor_returns_200() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-conflict-3").await;
    let tree_id = insert_tree_with_sensor(&app, "T-CONF-5", "eui-conflict-3").await;

    let update = serde_json::json!({
        "species": "Eiche",
        "number": "T-CONF-5",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Testbaum",
        "sensor_id": "eui-conflict-3"
    });
    let response = app
        .put_json(&format!("/api/v1/trees/{tree_id}"), &update)
        .await;

    assert_eq!(response.status().as_u16(), 200);
}

// -- Trees whose planting year has not arrived yet cannot carry a sensor --

fn future_year() -> i32 {
    chrono::Datelike::year(&chrono::Utc::now()) + 3
}

#[tokio::test]
async fn create_tree_planted_in_the_future_with_sensor_returns_422() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-future-1").await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-FUT-1",
        "planting_year": future_year(),
        "latitude": 53.56,
        "longitude": 9.98,
        "description": "Testbaum",
        "sensor_id": "eui-future-1"
    });
    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 422);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "tree.not_yet_planted");
}

#[tokio::test]
async fn create_tree_planted_in_the_future_without_sensor_succeeds() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "species": "Eiche",
        "number": "T-FUT-2",
        "planting_year": future_year(),
        "latitude": 53.56,
        "longitude": 9.98,
        "description": "Geplante Pflanzung"
    });
    let response = app.post_json("/api/v1/trees", &body).await;

    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn moving_a_sensor_carrying_tree_into_the_future_returns_422() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-future-2").await;
    let tree_id = insert_tree_with_sensor(&app, "T-FUT-3", "eui-future-2").await;

    let update = serde_json::json!({
        "species": "Eiche",
        "number": "T-FUT-3",
        "planting_year": future_year(),
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Test",
        "sensor_id": "eui-future-2"
    });
    let response = app
        .put_json(&format!("/api/v1/trees/{}", tree_id), &update)
        .await;

    assert_eq!(response.status().as_u16(), 422);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["code"], "tree.not_yet_planted");
}

// Dropping the sensor in the same edit is the way out of the rejection above.
#[tokio::test]
async fn moving_a_tree_into_the_future_while_dropping_its_sensor_succeeds() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-future-3").await;
    let tree_id = insert_tree_with_sensor(&app, "T-FUT-4", "eui-future-3").await;

    let update = serde_json::json!({
        "species": "Eiche",
        "number": "T-FUT-4",
        "planting_year": future_year(),
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Test",
        "sensor_id": null
    });
    let response = app
        .put_json(&format!("/api/v1/trees/{}", tree_id), &update)
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["sensor_id"].is_null());
}

#[tokio::test]
async fn list_trees_includes_cluster_name() {
    let app = spawn_app().await;

    let cluster = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Solitüde Strand",
                "address": "Strandweg 1",
                "description": "Testgruppe",
                "soil_condition": "Su3",
                "tree_ids": []
            }),
        )
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let cluster_id = cluster["id"].as_str().unwrap().to_string();

    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Eiche",
            "number": "T-100",
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "",
            "tree_cluster_id": cluster_id
        }),
    )
    .await;

    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Buche",
            "number": "T-200",
            "planting_year": 2021,
            "latitude": 53.56,
            "longitude": 9.98,
            "description": ""
        }),
    )
    .await;

    let body: serde_json::Value = app.get("/api/v1/trees").await.json().await.unwrap();
    let rows = body["data"].as_array().unwrap();

    let with_cluster = rows
        .iter()
        .find(|t| t["number"] == "T-100")
        .expect("tree with cluster is listed");
    assert_eq!(with_cluster["tree_cluster_name"], "Solitüde Strand");

    let without_cluster = rows
        .iter()
        .find(|t| t["number"] == "T-200")
        .expect("tree without cluster is listed");
    assert!(without_cluster.get("tree_cluster_name").is_none());
}

#[tokio::test]
async fn list_trees_filters_by_has_sensor() {
    let app = spawn_app().await;
    insert_sensor(&app, "eui-has-sensor-filter-1").await;
    insert_tree_with_sensor(&app, "T-300", "eui-has-sensor-filter-1").await;
    create_tree_with(&app, "T-301", 2020, None).await;

    let response = app.get("/api/v1/trees?has_sensor=true").await;
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-300");
    assert_eq!(body["pagination"]["total_records"], 1);

    let response = app.get("/api/v1/trees?has_sensor=false").await;
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["number"], "T-301");
    assert_eq!(body["pagination"]["total_records"], 1);

    let response = app.get("/api/v1/trees").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_trees_filters_by_cluster_id() {
    let app = spawn_app().await;

    let cluster = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Campusallee",
                "address": "Campusweg 1",
                "description": "Testgruppe",
                "soil_condition": "Su3",
                "tree_ids": []
            }),
        )
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let cluster_id = cluster["id"].as_str().unwrap().to_string();

    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Eiche",
            "number": "T-400",
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": "",
            "tree_cluster_id": cluster_id
        }),
    )
    .await;

    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Buche",
            "number": "T-500",
            "planting_year": 2021,
            "latitude": 53.56,
            "longitude": 9.98,
            "description": ""
        }),
    )
    .await;

    let body: serde_json::Value = app
        .get(&format!("/api/v1/trees?cluster_id={cluster_id}"))
        .await
        .json()
        .await
        .unwrap();

    let rows = body["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["number"], "T-400");
    assert_eq!(body["pagination"]["total_records"], 1);
}

async fn seed_sortable_trees(app: &TestApp) {
    for (number, species, year) in [
        ("T-001", "Betula pendula", 2020),
        ("T-002", "Acer platanoides", 2019),
        ("T-003", "Carpinus betulus", 2021),
    ] {
        app.post_json(
            "/api/v1/trees",
            &serde_json::json!({
                "species": species,
                "number": number,
                "planting_year": year,
                "latitude": 53.55,
                "longitude": 9.99,
                "description": ""
            }),
        )
        .await;
    }
}

fn numbers(body: &serde_json::Value) -> Vec<String> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["number"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn list_trees_sorts_by_number_ascending_by_default() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app.get("/api/v1/trees").await.json().await.unwrap();

    assert_eq!(numbers(&body), vec!["T-001", "T-002", "T-003"]);
}

#[tokio::test]
async fn list_trees_sorts_by_species_descending() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=species&order=desc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-003", "T-001", "T-002"]);
}

#[tokio::test]
async fn list_trees_sorts_by_planting_year_ascending() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=planting_year&order=asc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-002", "T-001", "T-003"]);
}

#[tokio::test]
async fn list_trees_sorts_by_status_with_critical_first() {
    let app = spawn_app().await;

    for (number, status) in [("T-010", "good"), ("T-011", "bad"), ("T-012", "moderate")] {
        let created: serde_json::Value = app
            .post_json(
                "/api/v1/trees",
                &serde_json::json!({
                    "species": "Eiche",
                    "number": number,
                    "planting_year": 2020,
                    "latitude": 53.55,
                    "longitude": 9.99,
                    "description": ""
                }),
            )
            .await
            .json()
            .await
            .unwrap();
        app.set_watering_status(created["id"].as_str().unwrap(), status)
            .await;
    }

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=status&order=asc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-011", "T-012", "T-010"]);
}

#[tokio::test]
async fn list_trees_sorts_by_cluster_name_with_unassigned_last() {
    let app = spawn_app().await;

    let bahnhofsviertel = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Bahnhofsviertel",
                "address": "Bahnhofstraße 1",
                "description": "Testgruppe",
                "soil_condition": "Su3",
                "tree_ids": []
            }),
        )
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let bahnhofsviertel_id = bahnhofsviertel["id"].as_str().unwrap().to_string();

    let alsterufer = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Alsterufer",
                "address": "Alsterweg 1",
                "description": "Testgruppe",
                "soil_condition": "Su3",
                "tree_ids": []
            }),
        )
        .await
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let alsterufer_id = alsterufer["id"].as_str().unwrap().to_string();

    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Eiche",
            "number": "T-020",
            "planting_year": 2020,
            "latitude": 53.55,
            "longitude": 9.99,
            "description": ""
        }),
    )
    .await;
    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Buche",
            "number": "T-021",
            "planting_year": 2020,
            "latitude": 53.56,
            "longitude": 9.98,
            "description": "",
            "tree_cluster_id": bahnhofsviertel_id
        }),
    )
    .await;
    app.post_json(
        "/api/v1/trees",
        &serde_json::json!({
            "species": "Linde",
            "number": "T-022",
            "planting_year": 2020,
            "latitude": 53.57,
            "longitude": 9.97,
            "description": "",
            "tree_cluster_id": alsterufer_id
        }),
    )
    .await;

    let asc: serde_json::Value = app
        .get("/api/v1/trees?sort=cluster&order=asc")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(numbers(&asc), vec!["T-022", "T-021", "T-020"]);

    let desc: serde_json::Value = app
        .get("/api/v1/trees?sort=cluster&order=desc")
        .await
        .json()
        .await
        .unwrap();
    // NULLS LAST holds in both directions, so the tree without a cluster stays
    // at the end rather than jumping to the front.
    assert_eq!(numbers(&desc), vec!["T-021", "T-022", "T-020"]);
}

#[tokio::test]
async fn list_trees_sorts_by_last_watered_in_both_directions() {
    let app = spawn_app().await;

    for (number, watered) in [
        ("T-030", Some("2026-02-15T08:00:00Z")),
        ("T-031", Some("2026-01-10T08:00:00Z")),
        ("T-032", Some("2026-03-20T08:00:00Z")),
        ("T-033", None),
    ] {
        let created: serde_json::Value = app
            .post_json(
                "/api/v1/trees",
                &serde_json::json!({
                    "species": "Eiche",
                    "number": number,
                    "planting_year": 2020,
                    "latitude": 53.55,
                    "longitude": 9.99,
                    "description": ""
                }),
            )
            .await
            .json()
            .await
            .unwrap();
        if let Some(watered_at) = watered {
            app.set_last_watered(created["id"].as_str().unwrap(), watered_at)
                .await;
        }
    }

    let asc: serde_json::Value = app
        .get("/api/v1/trees?sort=last_watered&order=asc")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(numbers(&asc), vec!["T-031", "T-030", "T-032", "T-033"]);

    let desc: serde_json::Value = app
        .get("/api/v1/trees?sort=last_watered&order=desc")
        .await
        .json()
        .await
        .unwrap();
    // T-033 (never watered) stays last in both directions: NULLS LAST.
    assert_eq!(numbers(&desc), vec!["T-032", "T-030", "T-031", "T-033"]);
}

#[tokio::test]
async fn list_trees_sorts_by_number_descending() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=number&order=desc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-003", "T-002", "T-001"]);
}

#[tokio::test]
async fn list_trees_sorts_by_status_descending() {
    let app = spawn_app().await;

    for (number, status) in [("T-040", "good"), ("T-041", "bad"), ("T-042", "moderate")] {
        let created: serde_json::Value = app
            .post_json(
                "/api/v1/trees",
                &serde_json::json!({
                    "species": "Eiche",
                    "number": number,
                    "planting_year": 2020,
                    "latitude": 53.55,
                    "longitude": 9.99,
                    "description": ""
                }),
            )
            .await
            .json()
            .await
            .unwrap();
        app.set_watering_status(created["id"].as_str().unwrap(), status)
            .await;
    }

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=status&order=desc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-040", "T-042", "T-041"]);
}

#[tokio::test]
async fn list_trees_sorts_by_planting_year_descending() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=planting_year&order=desc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-003", "T-001", "T-002"]);
}

#[tokio::test]
async fn list_trees_sorts_by_species_ascending() {
    let app = spawn_app().await;
    seed_sortable_trees(&app).await;

    let body: serde_json::Value = app
        .get("/api/v1/trees?sort=species&order=asc")
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(numbers(&body), vec!["T-002", "T-001", "T-003"]);
}

#[tokio::test]
async fn list_trees_keeps_a_stable_order_across_pages() {
    let app = spawn_app().await;

    for index in 0..6 {
        app.post_json(
            "/api/v1/trees",
            &serde_json::json!({
                "species": "Quercus robur",
                "number": format!("T-{index:03}"),
                "planting_year": 2020,
                "latitude": 53.55,
                "longitude": 9.99,
                "description": ""
            }),
        )
        .await;
    }

    let first: serde_json::Value = app
        .get("/api/v1/trees?sort=species&order=asc&page=1&per_page=3")
        .await
        .json()
        .await
        .unwrap();
    let second: serde_json::Value = app
        .get("/api/v1/trees?sort=species&order=asc&page=2&per_page=3")
        .await
        .json()
        .await
        .unwrap();

    let mut seen = numbers(&first);
    seen.extend(numbers(&second));
    seen.sort();
    seen.dedup();

    assert_eq!(
        seen.len(),
        6,
        "every tree appears exactly once across both pages"
    );
}

#[tokio::test]
async fn list_trees_rejects_an_unknown_sort_field() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees?sort=height").await;

    assert_eq!(response.status().as_u16(), 400);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn list_trees_rejects_an_unknown_sort_order() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/trees?order=sideways").await;

    assert_eq!(response.status().as_u16(), 400);
}
