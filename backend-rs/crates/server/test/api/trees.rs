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
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES
            ($1, 2020, 'Eiche',  'T-001',    54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'a'),
            ($2, 2021, 'Buche',  'T-50%STR', 54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'b'),
            ($3, 2022, 'Ahorn',  'T-1000',   54.79, 9.44, ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326), 'c')"#,
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
