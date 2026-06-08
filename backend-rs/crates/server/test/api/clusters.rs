use crate::helpers::{self, spawn_app};

#[tokio::test]
async fn list_clusters_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/clusters").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_clusters_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/clusters").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_clusters_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/clusters/{}", uuid::Uuid::now_v7()))
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_cluster_returns_201() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "name": "Stadtpark",
        "address": "Parkweg 1",
        "description": "Cluster im Stadtpark",
        "soil_condition": "sandig",
        "tree_ids": []
    });

    let response = app.post_json("/api/v1/clusters", &body).await;

    assert_eq!(response.status().as_u16(), 201);

    let cluster: serde_json::Value = response.json().await.unwrap();
    assert_eq!(cluster["name"], "Stadtpark");
    assert_eq!(cluster["address"], "Parkweg 1");
    assert_eq!(cluster["soil_condition"], "sandig");
    assert_eq!(cluster["trees"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_cluster_with_trees_links_them() {
    let app = spawn_app().await;

    let tree_id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES ($1, 2020, 'Eiche', 'T-001', 53.55, 9.99, ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test')"#,
        tree_id,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let body = serde_json::json!({
        "name": "Stadtpark",
        "address": "Parkweg 1",
        "description": "Cluster mit Baum",
        "soil_condition": "lehmig",
        "tree_ids": [tree_id]
    });

    let response = app.post_json("/api/v1/clusters", &body).await;

    assert_eq!(response.status().as_u16(), 201);

    let cluster: serde_json::Value = response.json().await.unwrap();
    assert_eq!(cluster["trees"].as_array().unwrap().len(), 1);
    assert_eq!(cluster["trees"][0]["id"], tree_id.to_string());
}

#[tokio::test]
async fn get_cluster_returns_full_response() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "name": "Marktplatz",
        "address": "Markt 1",
        "description": "Cluster am Markt",
        "soil_condition": "tonig",
        "tree_ids": []
    });

    let create_resp = app.post_json("/api/v1/clusters", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let response = app.get(&format!("/api/v1/clusters/{}", id)).await;

    assert_eq!(response.status().as_u16(), 200);

    let cluster: serde_json::Value = response.json().await.unwrap();
    assert_eq!(cluster["name"], "Marktplatz");
    assert_eq!(cluster["description"], "Cluster am Markt");
}

#[tokio::test]
async fn update_cluster_changes_name() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "name": "Alt",
        "address": "Strasse 1",
        "description": "Beschreibung",
        "soil_condition": "sandig",
        "tree_ids": []
    });

    let create_resp = app.post_json("/api/v1/clusters", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let update_body = serde_json::json!({
        "name": "Neu",
        "address": "Strasse 1",
        "description": "Beschreibung",
        "soil_condition": "sandig",
        "tree_ids": []
    });

    let response = app
        .put_json(&format!("/api/v1/clusters/{}", id), &update_body)
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let cluster: serde_json::Value = response.json().await.unwrap();
    assert_eq!(cluster["name"], "Neu");
}

#[tokio::test]
async fn delete_cluster_returns_204() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "name": "Loeschen",
        "address": "Weg 1",
        "description": "Wird geloescht",
        "soil_condition": "schluffig",
        "tree_ids": []
    });

    let create_resp = app.post_json("/api/v1/clusters", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let response = app.delete(&format!("/api/v1/clusters/{}", id)).await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get(&format!("/api/v1/clusters/{}", id)).await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_cluster_unlinks_trees_and_keeps_them_alive() {
    let app = spawn_app().await;

    let tree_1 = uuid::Uuid::now_v7();
    let tree_2 = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES
            ($1, 2020, 'Eiche', 'T-DEL-1', 53.55, 9.99, ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'A'),
            ($2, 2021, 'Linde', 'T-DEL-2', 53.56, 9.98, ST_SetSRID(ST_MakePoint(9.98, 53.56), 4326), 'B')"#,
        tree_1,
        tree_2,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let tree_ids: Vec<uuid::Uuid> = sqlx::query_scalar!(
        "SELECT id FROM trees WHERE number IN ('T-DEL-1', 'T-DEL-2') ORDER BY number"
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(tree_ids.len(), 2);

    let body = serde_json::json!({
        "name": "Cluster mit Baeumen",
        "address": "Allee 1",
        "description": "Wird geloescht, Baeume bleiben",
        "soil_condition": "lehmig",
        "tree_ids": tree_ids,
    });

    let create_resp = app.post_json("/api/v1/clusters", &body).await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let cluster_id = created["id"].as_str().unwrap();

    let response = app
        .delete(&format!("/api/v1/clusters/{}", cluster_id))
        .await;
    assert_eq!(response.status().as_u16(), 204);

    let remaining: Vec<(uuid::Uuid, Option<uuid::Uuid>)> = sqlx::query_as(
        "SELECT id, tree_cluster_id FROM trees WHERE number IN ('T-DEL-1', 'T-DEL-2') ORDER BY number",
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap();

    assert_eq!(remaining.len(), 2, "trees must survive cluster delete");
    for (_id, cluster) in &remaining {
        assert!(
            cluster.is_none(),
            "tree_cluster_id must be cleared after cluster delete"
        );
    }
}

#[tokio::test]
async fn list_clusters_respects_pagination() {
    let app = spawn_app().await;

    for i in 1..=5 {
        let body = serde_json::json!({
            "name": format!("Cluster {}", i),
            "address": format!("Adresse {}", i),
            "description": "Test",
            "soil_condition": "sandig",
            "tree_ids": []
        });
        app.post_json("/api/v1/clusters", &body).await;
    }

    let response = app.get("/api/v1/clusters?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
    assert!(body["pagination"]["next_page"].as_u64().is_some());
}

// -- Event handler integration tests --

async fn insert_tree_at(app: &helpers::TestApp, lat: f64, lng: f64, number: &str) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude, geometry, description)
        VALUES ($1, 2020, 'Eiche', $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), 'Test')"#,
        id,
        number,
        lat,
        lng,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

async fn insert_region_covering(
    app: &helpers::TestApp,
    name: &str,
    lat: f64,
    lng: f64,
) -> uuid::Uuid {
    let wkt = format!(
        "MULTIPOLYGON((({} {}, {} {}, {} {}, {} {}, {} {})))",
        lng - 0.01,
        lat - 0.01,
        lng + 0.01,
        lat - 0.01,
        lng + 0.01,
        lat + 0.01,
        lng - 0.01,
        lat + 0.01,
        lng - 0.01,
        lat - 0.01,
    );
    let id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO regions (id, name, geometry)
        VALUES ($1, $2, ST_SetSRID(ST_GeomFromText($3), 4326))"#,
        id,
        name,
        wkt,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn create_cluster_with_trees_computes_center_point() {
    let app = spawn_app().await;

    let tree_id = insert_tree_at(&app, 53.55, 9.99, "T-CENTER").await;

    let body = serde_json::json!({
        "name": "Center Test",
        "address": "Testweg",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [tree_id]
    });

    let response = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(response.status().as_u16(), 201);

    let cluster: serde_json::Value = response.json().await.unwrap();
    let id = cluster["id"].as_str().unwrap();

    // Fetch again to see the recalculated values
    let get_resp = app.get(&format!("/api/v1/clusters/{}", id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();

    let lat = cluster["latitude"].as_f64().unwrap();
    let lng = cluster["longitude"].as_f64().unwrap();
    assert!((lat - 53.55).abs() < 0.001);
    assert!((lng - 9.99).abs() < 0.001);
}

#[tokio::test]
async fn create_cluster_with_trees_assigns_region() {
    let app = spawn_app().await;

    let _region_id = insert_region_covering(&app, "Altstadt", 53.55, 9.99).await;
    let tree_id = insert_tree_at(&app, 53.55, 9.99, "T-REGION").await;

    let body = serde_json::json!({
        "name": "Region Test",
        "address": "Regionweg",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [tree_id]
    });

    let response = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(response.status().as_u16(), 201);

    let cluster: serde_json::Value = response.json().await.unwrap();
    let id = cluster["id"].as_str().unwrap();

    let get_resp = app.get(&format!("/api/v1/clusters/{}", id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();

    assert!(cluster["region"].is_object(), "region should be assigned");
    assert_eq!(cluster["region"]["name"], "Altstadt");
}

#[tokio::test]
async fn tree_create_triggers_cluster_recalculation() {
    let app = spawn_app().await;

    // Create empty cluster
    let cluster_body = serde_json::json!({
        "name": "Recalc Test",
        "address": "Test",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": []
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // Cluster should have no coordinates yet
    assert_eq!(cluster["latitude"].as_f64().unwrap(), 0.0);

    // Create tree linked to cluster via API → triggers TreeCreated event
    let tree_body = serde_json::json!({
        "species": "Buche",
        "number": "T-RECALC",
        "planting_year": 2020,
        "latitude": 53.56,
        "longitude": 10.01,
        "description": "Test",
        "tree_cluster_id": cluster_id
    });
    let tree_resp = app.post_json("/api/v1/trees", &tree_body).await;
    assert_eq!(tree_resp.status().as_u16(), 201);

    // Cluster should now have coordinates from the tree
    let get_resp = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();

    let lat = cluster["latitude"].as_f64().unwrap();
    let lng = cluster["longitude"].as_f64().unwrap();
    assert!(
        (lat - 53.56).abs() < 0.001,
        "latitude should be ~53.56, got {lat}"
    );
    assert!(
        (lng - 10.01).abs() < 0.001,
        "longitude should be ~10.01, got {lng}"
    );
}

#[tokio::test]
async fn tree_create_triggers_region_assignment() {
    let app = spawn_app().await;

    insert_region_covering(&app, "Neustadt", 53.56, 10.01).await;

    // Create empty cluster
    let cluster_body = serde_json::json!({
        "name": "Region Assign",
        "address": "Test",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": []
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // No region yet
    assert!(cluster["region"].is_null());

    // Create tree inside Neustadt region → TreeCreated → recalc → region assigned
    let tree_body = serde_json::json!({
        "species": "Ahorn",
        "number": "T-ASSIGN",
        "planting_year": 2019,
        "latitude": 53.56,
        "longitude": 10.01,
        "description": "Test",
        "tree_cluster_id": cluster_id
    });
    app.post_json("/api/v1/trees", &tree_body).await;

    let get_resp = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();

    assert!(
        cluster["region"].is_object(),
        "region should be assigned after tree creation"
    );
    assert_eq!(cluster["region"]["name"], "Neustadt");
}

#[tokio::test]
async fn tree_delete_triggers_cluster_coordinate_reset() {
    let app = spawn_app().await;

    // Create cluster with one tree
    let tree_id = insert_tree_at(&app, 53.55, 9.99, "T-DELRESET").await;

    let cluster_body = serde_json::json!({
        "name": "Delete Reset",
        "address": "Test",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [tree_id]
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // Verify cluster has coordinates
    let get_resp = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();
    assert!((cluster["latitude"].as_f64().unwrap() - 53.55).abs() < 0.001);

    // Delete the tree → TreeDeleted event → recalc → no trees → coordinates reset
    app.delete(&format!("/api/v1/trees/{}", tree_id)).await;

    let get_resp = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let cluster: serde_json::Value = get_resp.json().await.unwrap();

    // With no trees, center_point fails gracefully → coordinates = 0.0
    assert_eq!(cluster["latitude"].as_f64().unwrap(), 0.0);
    assert_eq!(cluster["longitude"].as_f64().unwrap(), 0.0);
}

#[tokio::test]
async fn tree_move_to_other_cluster_recalculates_both() {
    let app = spawn_app().await;

    // Create two clusters, each with one tree
    let tree_a_id = insert_tree_at(&app, 53.50, 9.90, "T-MOVE-A").await;
    let tree_b_id = insert_tree_at(&app, 53.60, 10.10, "T-MOVE-B").await;

    let cluster_a_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Cluster A",
                "address": "A",
                "description": "A",
                "soil_condition": "sandig",
                "tree_ids": [tree_a_id]
            }),
        )
        .await;
    let cluster_a: serde_json::Value = cluster_a_resp.json().await.unwrap();
    let cluster_a_id = cluster_a["id"].as_str().unwrap();

    let cluster_b_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Cluster B",
                "address": "B",
                "description": "B",
                "soil_condition": "sandig",
                "tree_ids": [tree_b_id]
            }),
        )
        .await;
    let cluster_b: serde_json::Value = cluster_b_resp.json().await.unwrap();
    let cluster_b_id = cluster_b["id"].as_str().unwrap();

    // Verify initial state: both clusters have coordinates from their trees
    let a = app.get(&format!("/api/v1/clusters/{}", cluster_a_id)).await;
    let a_body: serde_json::Value = a.json().await.unwrap();
    assert!((a_body["latitude"].as_f64().unwrap() - 53.50).abs() < 0.001);

    // Move tree_a from cluster A to cluster B via tree update
    let update_body = serde_json::json!({
        "species": "Eiche",
        "number": "T-MOVE-A",
        "planting_year": 2020,
        "latitude": 53.50,
        "longitude": 9.90,
        "description": "Test",
        "tree_cluster_id": cluster_b_id
    });
    let update_resp = app
        .put_json(&format!("/api/v1/trees/{}", tree_a_id), &update_body)
        .await;
    assert_eq!(update_resp.status().as_u16(), 200);

    // Cluster A: no more trees → coordinates reset
    let a_after = app.get(&format!("/api/v1/clusters/{}", cluster_a_id)).await;
    let a_after_body: serde_json::Value = a_after.json().await.unwrap();
    assert_eq!(
        a_after_body["latitude"].as_f64().unwrap(),
        0.0,
        "Cluster A should have no coordinates after losing its tree"
    );

    // Cluster B: now has two trees → center recalculated
    let b_after = app.get(&format!("/api/v1/clusters/{}", cluster_b_id)).await;
    let b_after_body: serde_json::Value = b_after.json().await.unwrap();
    let b_lat = b_after_body["latitude"].as_f64().unwrap();
    // Center of (53.50, 9.90) and (53.60, 10.10) is roughly (53.55, 10.00)
    assert!(
        (b_lat - 53.55).abs() < 0.01,
        "Cluster B center should be ~53.55, got {b_lat}"
    );
}

#[tokio::test]
async fn cluster_update_tree_ids_recalculates_center() {
    let app = spawn_app().await;

    // Create cluster with two trees at different positions
    let tree1_id = insert_tree_at(&app, 53.50, 9.90, "T-REM-1").await;
    let tree2_id = insert_tree_at(&app, 53.60, 10.10, "T-REM-2").await;

    let cluster_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Remove Test",
                "address": "X",
                "description": "X",
                "soil_condition": "sandig",
                "tree_ids": [tree1_id, tree2_id]
            }),
        )
        .await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // Center should be ~(53.55, 10.0) with both trees
    let before = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let before_body: serde_json::Value = before.json().await.unwrap();
    let before_lat = before_body["latitude"].as_f64().unwrap();
    assert!((before_lat - 53.55).abs() < 0.01);

    // Remove tree1 via cluster update → only tree2 remains
    let update_body = serde_json::json!({
        "name": "Remove Test",
        "address": "X",
        "description": "X",
        "soil_condition": "sandig",
        "tree_ids": [tree2_id]
    });
    app.put_json(&format!("/api/v1/clusters/{}", cluster_id), &update_body)
        .await;

    // Center should now be at tree2's position (53.60, 10.10)
    let after = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let after_body: serde_json::Value = after.json().await.unwrap();
    let after_lat = after_body["latitude"].as_f64().unwrap();
    assert!(
        (after_lat - 53.60).abs() < 0.01,
        "Center should shift to remaining tree at 53.60, got {after_lat}"
    );
}

// -- ClusterStatusAggregator (sensor-equipped tree status → cluster status) --

async fn insert_sensor(app: &helpers::TestApp, id: &str) {
    let model_id = app.ecodrizzler_model_id().await;
    sqlx::query!(
        r#"INSERT INTO sensors (id, activated_at, type, model_id)
        VALUES ($1, NOW(), 'lorawan', $2)"#,
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

async fn insert_tree_with(
    app: &helpers::TestApp,
    number: &str,
    sensor_id: Option<&str>,
    watering_status: &str,
) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, planting_year, species, number, latitude, longitude,
                              geometry, description, sensor_id, watering_status)
        VALUES ($1, 2020, 'Eiche', $2, 53.55, 9.99,
                ST_SetSRID(ST_MakePoint(9.99, 53.55), 4326), 'Test', $3,
                $4::text::watering_status)"#,
        id,
        number,
        sensor_id,
        watering_status,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn cluster_status_takes_majority_among_sensor_equipped_trees() {
    let app = spawn_app().await;

    insert_sensor(&app, "s-good-1").await;
    insert_sensor(&app, "s-good-2").await;
    insert_sensor(&app, "s-bad-1").await;
    let t_good_1 = insert_tree_with(&app, "T-AG-1", Some("s-good-1"), "good").await;
    let t_good_2 = insert_tree_with(&app, "T-AG-2", Some("s-good-2"), "good").await;
    let t_bad = insert_tree_with(&app, "T-AG-3", Some("s-bad-1"), "bad").await;
    let t_sensorless = insert_tree_with(&app, "T-AG-4", None, "bad").await;

    let body = serde_json::json!({
        "name": "Status Aggregator",
        "address": "Test",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [t_good_1, t_good_2, t_bad, t_sensorless],
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    let cluster: serde_json::Value = resp.json().await.unwrap();
    let cid = cluster["id"].as_str().unwrap();

    let after = app.get(&format!("/api/v1/clusters/{}", cid)).await;
    let after: serde_json::Value = after.json().await.unwrap();
    assert_eq!(
        after["watering_status"], "good",
        "majority among sensor-equipped trees is good (2/3); sensorless tree must be ignored"
    );
}

#[tokio::test]
async fn cluster_status_is_unknown_with_only_sensorless_trees() {
    let app = spawn_app().await;

    let t1 = insert_tree_with(&app, "T-NS-1", None, "good").await;
    let t2 = insert_tree_with(&app, "T-NS-2", None, "bad").await;

    let body = serde_json::json!({
        "name": "Sensorless Cluster",
        "address": "Test",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [t1, t2],
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    let cluster: serde_json::Value = resp.json().await.unwrap();
    let cid = cluster["id"].as_str().unwrap();

    let after = app.get(&format!("/api/v1/clusters/{}", cid)).await;
    let after: serde_json::Value = after.json().await.unwrap();
    assert_eq!(after["watering_status"], "unknown");
}

#[tokio::test]
async fn attaching_sensor_to_tree_recalculates_cluster_status() {
    let app = spawn_app().await;

    insert_sensor(&app, "s-attach").await;
    let tree_id = insert_tree_with(&app, "T-ATT", None, "good").await;

    let cluster_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Attach Test",
                "address": "Test",
                "description": "Test",
                "soil_condition": "sandig",
                "tree_ids": [tree_id],
            }),
        )
        .await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cid = cluster["id"].as_str().unwrap();

    let before = app.get(&format!("/api/v1/clusters/{}", cid)).await;
    let before: serde_json::Value = before.json().await.unwrap();
    assert_eq!(
        before["watering_status"], "unknown",
        "no sensor-equipped trees yet"
    );

    let update = serde_json::json!({
        "species": "Eiche",
        "number": "T-ATT",
        "planting_year": 2020,
        "latitude": 53.55,
        "longitude": 9.99,
        "description": "Test",
        "tree_cluster_id": cid,
        "sensor_id": "s-attach",
    });
    let upd = app
        .put_json(&format!("/api/v1/trees/{}", tree_id), &update)
        .await;
    assert_eq!(upd.status().as_u16(), 200);

    let after = app.get(&format!("/api/v1/clusters/{}", cid)).await;
    let after: serde_json::Value = after.json().await.unwrap();
    assert_eq!(
        after["watering_status"], "good",
        "after sensor attach, the now-counted tree status drives cluster status"
    );
}

#[tokio::test]
async fn list_cluster_markers_returns_empty_when_none_exist() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/clusters/markers").await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_cluster_markers_excludes_clusters_without_centroid() {
    let app = spawn_app().await;

    let cluster_body = serde_json::json!({
        "name": "Empty Park",
        "address": "Nowhere 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [],
    });
    let resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app.get("/api/v1/clusters/markers").await;
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_cluster_markers_includes_cluster_with_trees() {
    let app = spawn_app().await;

    let cluster_body = serde_json::json!({
        "name": "Stadtpark",
        "address": "Parkweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [],
    });
    let cluster_resp = app.post_json("/api/v1/clusters", &cluster_body).await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    let tree_body = serde_json::json!({
        "species": "Eiche",
        "number": "T-CL-001",
        "planting_year": 2020,
        "latitude": 54.79,
        "longitude": 9.44,
        "description": "x",
        "tree_cluster_id": cluster_id
    });
    let resp = app.post_json("/api/v1/trees", &tree_body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app.get("/api/v1/clusters/markers").await;
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Stadtpark");
    assert_eq!(data[0]["tree_count"], 1);
}

async fn create_cluster_named(app: &helpers::TestApp, name: &str) -> String {
    let body = serde_json::json!({
        "name": name,
        "address": "Parkweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": []
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(resp.status().as_u16(), 201);
    let cluster: serde_json::Value = resp.json().await.unwrap();
    cluster["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn list_clusters_filters_by_watering_status() {
    let app = spawn_app().await;
    create_cluster_named(&app, "Cluster A").await;
    create_cluster_named(&app, "Cluster B").await;

    // freshly created clusters have status "unknown"
    let response = app.get("/api/v1/clusters?watering_status=unknown").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    let response = app.get("/api/v1/clusters?watering_status=good").await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_clusters_accepts_repeated_watering_statuses() {
    let app = spawn_app().await;
    create_cluster_named(&app, "Cluster A").await;

    let response = app
        .get("/api/v1/clusters?watering_status=good&watering_status=unknown")
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_clusters_filters_by_region() {
    let app = spawn_app().await;
    let cluster_a = create_cluster_named(&app, "Cluster A").await;
    create_cluster_named(&app, "Cluster B").await;

    // Region assignment normally happens via spatial lookup; set it directly
    // to keep the test deterministic.
    let region_id = uuid::Uuid::now_v7();
    sqlx::query("INSERT INTO regions (id, name) VALUES ($1, $2)")
        .bind(region_id)
        .bind("Mürwik")
        .execute(&app.db_pool)
        .await
        .unwrap();
    sqlx::query("UPDATE tree_clusters SET region_id = $1 WHERE id = $2")
        .bind(region_id)
        .bind(uuid::Uuid::parse_str(&cluster_a).unwrap())
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app
        .get(&format!("/api/v1/clusters?region={region_id}"))
        .await;
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Cluster A");

    let other = uuid::Uuid::now_v7();
    let response = app.get(&format!("/api/v1/clusters?region={other}")).await;
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn tree_position_update_recalculates_cluster_center() {
    let app = spawn_app().await;

    // Create cluster with one tree at (53.50, 9.90)
    let tree_id = insert_tree_at(&app, 53.50, 9.90, "T-POS").await;

    let cluster_resp = app
        .post_json(
            "/api/v1/clusters",
            &serde_json::json!({
                "name": "Position Test",
                "address": "X",
                "description": "X",
                "soil_condition": "sandig",
                "tree_ids": [tree_id]
            }),
        )
        .await;
    let cluster: serde_json::Value = cluster_resp.json().await.unwrap();
    let cluster_id = cluster["id"].as_str().unwrap();

    // Verify initial center at (53.50, 9.90)
    let before = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let before_body: serde_json::Value = before.json().await.unwrap();
    assert!((before_body["latitude"].as_f64().unwrap() - 53.50).abs() < 0.001);
    assert!((before_body["longitude"].as_f64().unwrap() - 9.90).abs() < 0.001);

    // Move tree to (53.60, 10.10) → TreeCoordinateChanged event → cluster recalc
    let update_body = serde_json::json!({
        "species": "Eiche",
        "number": "T-POS",
        "planting_year": 2020,
        "latitude": 53.60,
        "longitude": 10.10,
        "description": "Test",
        "tree_cluster_id": cluster_id
    });
    let update_resp = app
        .put_json(&format!("/api/v1/trees/{}", tree_id), &update_body)
        .await;
    assert_eq!(update_resp.status().as_u16(), 200);

    // Cluster center should now be at (53.60, 10.10)
    let after = app.get(&format!("/api/v1/clusters/{}", cluster_id)).await;
    let after_body: serde_json::Value = after.json().await.unwrap();
    let after_lat = after_body["latitude"].as_f64().unwrap();
    let after_lng = after_body["longitude"].as_f64().unwrap();
    assert!(
        (after_lat - 53.60).abs() < 0.001,
        "Cluster lat should be ~53.60, got {after_lat}"
    );
    assert!(
        (after_lng - 10.10).abs() < 0.001,
        "Cluster lng should be ~10.10, got {after_lng}"
    );
}

// -- Cluster boundary (convex hull) --

#[tokio::test]
async fn list_cluster_boundaries_returns_empty_when_none_exist() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/clusters/boundaries").await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_cluster_boundaries_returns_polygon_for_cluster_with_trees() {
    let app = spawn_app().await;

    // Three trees forming a triangle → a real convex hull.
    let t1 = insert_tree_at(&app, 53.550, 9.990, "T-BND-1").await;
    let t2 = insert_tree_at(&app, 53.560, 9.990, "T-BND-2").await;
    let t3 = insert_tree_at(&app, 53.555, 10.000, "T-BND-3").await;

    let body = serde_json::json!({
        "name": "Boundary Cluster",
        "address": "Hüllenweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [t1, t2, t3],
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app.get("/api/v1/clusters/boundaries").await;
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "one non-archived cluster with trees");

    assert_eq!(data[0]["name"], "Boundary Cluster");
    assert_eq!(data[0]["watering_status"], "unknown");
    // GeoJSON geometry: a buffered hull is always a Polygon.
    assert_eq!(data[0]["boundary"]["type"], "Polygon");
    let rings = data[0]["boundary"]["coordinates"].as_array().unwrap();
    assert!(
        !rings.is_empty(),
        "polygon must have at least an exterior ring"
    );
    assert!(
        rings[0].as_array().unwrap().len() >= 4,
        "a polygon ring is closed and has >= 4 positions"
    );
}

#[tokio::test]
async fn list_cluster_boundaries_handles_single_tree_as_buffered_circle() {
    let app = spawn_app().await;

    let t1 = insert_tree_at(&app, 53.55, 9.99, "T-BND-SINGLE").await;
    let body = serde_json::json!({
        "name": "Single Tree Cluster",
        "address": "Einzelweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [t1],
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    assert_eq!(resp.status().as_u16(), 201);

    let resp = app.get("/api/v1/clusters/boundaries").await;
    let json: serde_json::Value = resp.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    // Buffering a single point yields a Polygon (circle), not a Point.
    assert_eq!(data[0]["boundary"]["type"], "Polygon");
}

#[tokio::test]
async fn list_cluster_boundaries_excludes_archived_clusters() {
    let app = spawn_app().await;

    let t1 = insert_tree_at(&app, 53.550, 9.990, "T-BND-ARCH-1").await;
    let t2 = insert_tree_at(&app, 53.560, 9.990, "T-BND-ARCH-2").await;
    let t3 = insert_tree_at(&app, 53.555, 10.000, "T-BND-ARCH-3").await;

    let body = serde_json::json!({
        "name": "Archived Cluster",
        "address": "Archivweg 1",
        "description": "Test",
        "soil_condition": "sandig",
        "tree_ids": [t1, t2, t3],
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    let cluster: serde_json::Value = resp.json().await.unwrap();
    let id = uuid::Uuid::parse_str(cluster["id"].as_str().unwrap()).unwrap();

    sqlx::query("UPDATE tree_clusters SET archived = true WHERE id = $1")
        .bind(id)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let resp = app.get("/api/v1/clusters/boundaries").await;
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        json["data"].as_array().unwrap().len(),
        0,
        "archived clusters must be excluded from boundaries"
    );
}
