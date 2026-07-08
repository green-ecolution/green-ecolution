use crate::helpers::{self, spawn_app};

async fn create_transporter(app: &helpers::TestApp) -> serde_json::Value {
    let body = serde_json::json!({
        "number_plate": "FL-GE 100",
        "description": "Giesswagen",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
    });
    let resp = app.post_json("/api/v1/vehicles", &body).await;
    resp.json().await.unwrap()
}

async fn create_trailer(app: &helpers::TestApp) -> serde_json::Value {
    let body = serde_json::json!({
        "number_plate": "FL-GE 200",
        "description": "Anhaenger",
        "water_capacity": 3000.0,
        "model": "Trailer X",
        "status": "available",
        "type": "trailer",
        "driving_license": "BE",
        "height": 2.0, "width": 2.0, "length": 5.0, "weight": 3000.0
    });
    let resp = app.post_json("/api/v1/vehicles", &body).await;
    resp.json().await.unwrap()
}

async fn create_cluster(app: &helpers::TestApp) -> serde_json::Value {
    let body = serde_json::json!({
        "name": "Testcluster",
        "address": "Testweg 1",
        "description": "Test",
        "soil_condition": "Su3",
        "tree_ids": []
    });
    let resp = app.post_json("/api/v1/clusters", &body).await;
    resp.json().await.unwrap()
}

fn plan_body(transporter_id: &str, cluster_ids: Vec<&str>) -> serde_json::Value {
    serde_json::json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Bewaesserung Innenstadt",
        "transporter_id": transporter_id,
        "tree_cluster_ids": cluster_ids,
        "user_ids": []
    })
}

#[tokio::test]
async fn list_watering_plans_returns_200() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/watering-plans").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn list_watering_plans_returns_empty_list() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/watering-plans").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn get_watering_plan_returns_404_for_nonexistent_id() {
    let app = spawn_app().await;

    let response = app
        .get(&format!("/api/v1/watering-plans/{}", uuid::Uuid::now_v7()))
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_watering_plan_returns_201() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let response = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["description"], "Bewaesserung Innenstadt");
    assert_eq!(plan["status"], "planned");
    assert_eq!(plan["transporter"]["id"], tid);
}

#[tokio::test]
async fn create_watering_plan_with_clusters() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();

    let response = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![cid]))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["treeclusters"].as_array().unwrap().len(), 1);
    assert_eq!(plan["treeclusters"][0]["id"], cid);
}

#[tokio::test]
async fn create_watering_plan_with_trailer() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let trailer = create_trailer(&app).await;
    let trailer_id = trailer["id"].as_str().unwrap();

    let mut body = plan_body(tid, vec![]);
    body["trailer_id"] = serde_json::json!(trailer_id);

    let response = app.post_json("/api/v1/watering-plans", &body).await;

    assert_eq!(response.status().as_u16(), 201);

    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["transporter"]["id"], tid);
    assert!(plan["trailer"].is_object());
    assert_eq!(plan["trailer"]["id"], trailer_id);
}

#[tokio::test]
async fn create_watering_plan_with_invalid_date_returns_400() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let body = serde_json::json!({
        "date": "not-a-date",
        "description": "Test",
        "transporter_id": tid,
        "tree_cluster_ids": [],
        "user_ids": []
    });

    let response = app.post_json("/api/v1/watering-plans", &body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_watering_plan_returns_full_response() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let response = app.get(&format!("/api/v1/watering-plans/{}", id)).await;

    assert_eq!(response.status().as_u16(), 200);

    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["description"], "Bewaesserung Innenstadt");
    assert_eq!(plan["transporter"]["number_plate"], "FL-GE 100");
}

#[tokio::test]
async fn update_watering_plan_changes_description() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let update_body = serde_json::json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Aktualisiert",
        "status": "active",
        "transporter_id": tid,
        "tree_cluster_ids": [],
        "user_ids": [],
        "cancellation_note": ""
    });

    let response = app
        .put_json(&format!("/api/v1/watering-plans/{}", id), &update_body)
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["description"], "Aktualisiert");
    assert_eq!(plan["status"], "active");
}

#[tokio::test]
async fn delete_watering_plan_returns_204() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let response = app.delete(&format!("/api/v1/watering-plans/{}", id)).await;
    assert_eq!(response.status().as_u16(), 204);

    let get_resp = app.get(&format!("/api/v1/watering-plans/{}", id)).await;
    assert_eq!(get_resp.status().as_u16(), 404);
}

#[tokio::test]
async fn list_watering_plans_respects_pagination() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    for _ in 0..5 {
        app.post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
            .await;
    }

    let response = app.get("/api/v1/watering-plans?page=1&per_page=2").await;
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total_records"], 5);
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["total_pages"], 3);
}

#[tokio::test]
async fn list_watering_plans_includes_resolved_vehicles() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    app.post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;

    let response = app.get("/api/v1/watering-plans").await;
    let body: serde_json::Value = response.json().await.unwrap();

    let plan = &body["data"][0];
    assert_eq!(plan["transporter"]["number_plate"], "FL-GE 100");
}

fn update_body_with_status(
    transporter_id: &str,
    cluster_ids: Vec<&str>,
    status: &str,
    cancellation_note: &str,
    evaluation: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Bewaesserung Innenstadt",
        "status": status,
        "transporter_id": transporter_id,
        "tree_cluster_ids": cluster_ids,
        "user_ids": [],
        "cancellation_note": cancellation_note,
        "evaluation": evaluation,
    })
}

#[tokio::test]
async fn finish_watering_plan_propagates_last_watered_and_persists_evaluations() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();
    assert!(
        cluster["last_watered"].is_null(),
        "cluster should start without last_watered"
    );

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![cid]))
        .await;
    let plan: serde_json::Value = create_resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let activate = update_body_with_status(tid, vec![cid], "active", "", serde_json::json!([]));
    let activate_resp = app
        .put_json(&format!("/api/v1/watering-plans/{}", plan_id), &activate)
        .await;
    assert_eq!(activate_resp.status().as_u16(), 200);

    let finish = update_body_with_status(
        tid,
        vec![cid],
        "finished",
        "",
        serde_json::json!([{
            "watering_plan_id": plan_id,
            "tree_cluster_id": cid,
            "consumed_water": 1234.5
        }]),
    );
    let finish_resp = app
        .put_json(&format!("/api/v1/watering-plans/{}", plan_id), &finish)
        .await;
    assert_eq!(finish_resp.status().as_u16(), 200);

    let finished: serde_json::Value = finish_resp.json().await.unwrap();
    assert_eq!(finished["status"], "finished");
    let evals = finished["evaluation"].as_array().unwrap();
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0]["tree_cluster_id"], cid);
    assert_eq!(evals[0]["consumed_water"], 1234.5);

    let cluster_after = app.get(&format!("/api/v1/clusters/{}", cid)).await;
    let cluster_after: serde_json::Value = cluster_after.json().await.unwrap();
    assert!(
        cluster_after["last_watered"].is_string(),
        "cluster.last_watered must be set after plan finish, got {:?}",
        cluster_after["last_watered"]
    );
}

#[tokio::test]
async fn finish_without_evaluation_for_cluster_returns_400() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![cid]))
        .await;
    let plan: serde_json::Value = create_resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    app.put_json(
        &format!("/api/v1/watering-plans/{}", plan_id),
        &update_body_with_status(tid, vec![cid], "active", "", serde_json::json!([])),
    )
    .await;

    let finish_resp = app
        .put_json(
            &format!("/api/v1/watering-plans/{}", plan_id),
            &update_body_with_status(tid, vec![cid], "finished", "", serde_json::json!([])),
        )
        .await;
    assert_eq!(finish_resp.status().as_u16(), 400);
}

#[tokio::test]
async fn cancel_active_watering_plan_with_note_succeeds() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;
    let plan: serde_json::Value = create_resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    app.put_json(
        &format!("/api/v1/watering-plans/{}", plan_id),
        &update_body_with_status(tid, vec![], "active", "", serde_json::json!([])),
    )
    .await;

    let cancel_resp = app
        .put_json(
            &format!("/api/v1/watering-plans/{}", plan_id),
            &update_body_with_status(tid, vec![], "canceled", "Wetter", serde_json::json!([])),
        )
        .await;
    assert_eq!(cancel_resp.status().as_u16(), 200);
    let body: serde_json::Value = cancel_resp.json().await.unwrap();
    assert_eq!(body["status"], "canceled");
}

#[tokio::test]
async fn preview_route_returns_503_when_routing_disabled() {
    let app = spawn_app().await;

    let response = app
        .post_json(
            "/api/v1/watering-plans/route/preview",
            &serde_json::json!({
                "cluster_ids": ["0190a8e9-7c4f-7000-8000-000000000000"],
                "transporter_id": "0190a8e9-7c4f-7000-8000-000000000000"
            }),
        )
        .await;

    assert_eq!(response.status().as_u16(), 503);
    let body = response.text().await.unwrap_or_default();
    assert!(
        body.contains("routing"),
        "expected error body to mention routing, got: {body}"
    );
}

#[tokio::test]
async fn get_gpx_file_returns_503_when_routing_disabled() {
    let app = spawn_app().await;

    let response = app
        .get("/api/v1/watering-plans/0190a8e9-7c4f-7000-8000-000000000000/route/gpx")
        .await;

    assert_eq!(response.status().as_u16(), 503);
    let body = response.text().await.unwrap_or_default();
    assert!(
        body.contains("routing"),
        "expected error body to mention routing, got: {body}"
    );
}

#[tokio::test]
async fn watering_plan_roles_survive_unfavorable_vehicle_id_order() {
    let app = spawn_app().await;

    // Trailer first: its uuid v7 sorts before the transporter's, which used
    // to swap the roles in the positional ARRAY_AGG(DISTINCT ...) decoding.
    let trailer = create_trailer(&app).await;
    let trailer_id = trailer["id"].as_str().unwrap();
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();

    let mut body = plan_body(tid, vec![cid]);
    body["trailer_id"] = serde_json::json!(trailer_id);
    let resp = app.post_json("/api/v1/watering-plans", &body).await;
    assert_eq!(resp.status().as_u16(), 201);
    let plan: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(plan["transporter"]["id"], tid);
    assert_eq!(plan["trailer"]["id"], trailer_id);

    // Re-read must decode the same roles.
    let get = app
        .get(&format!(
            "/api/v1/watering-plans/{}",
            plan["id"].as_str().unwrap()
        ))
        .await;
    let got: serde_json::Value = get.json().await.unwrap();
    assert_eq!(got["transporter"]["id"], tid);
    assert_eq!(got["trailer"]["id"], trailer_id);
}

#[tokio::test]
async fn list_watering_plans_skips_plan_without_transporter() {
    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();
    let created = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![cid]))
        .await;
    assert_eq!(created.status().as_u16(), 201);

    // Data-integrity edge case: a plan without any vehicle join rows must not
    // take down the whole list endpoint.
    sqlx::query("INSERT INTO watering_plans (id, date, description, status) VALUES ($1, '2026-05-01', 'kaputt', 'planned')")
        .bind(uuid::Uuid::now_v7())
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.get("/api/v1/watering-plans").await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        1,
        "the transporter-less plan must be skipped, not panic the endpoint"
    );
}

#[tokio::test]
async fn finished_plan_keeps_consumed_water_across_save() {
    use domain::watering_plan::{WateringPlanReader, WateringPlanWriter};
    use server::infra::pg_watering_plan::PgWateringPlanRepository;

    let app = spawn_app().await;

    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let cluster = create_cluster(&app).await;
    let cid = cluster["id"].as_str().unwrap();

    let create_resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![cid]))
        .await;
    let plan: serde_json::Value = create_resp.json().await.unwrap();
    let plan_id = plan["id"].as_str().unwrap();

    let activate = update_body_with_status(tid, vec![cid], "active", "", serde_json::json!([]));
    app.put_json(&format!("/api/v1/watering-plans/{}", plan_id), &activate)
        .await;
    let finish = update_body_with_status(
        tid,
        vec![cid],
        "finished",
        "",
        serde_json::json!([{
            "watering_plan_id": plan_id,
            "tree_cluster_id": cid,
            "consumed_water": 42.5
        }]),
    );
    let finish_resp = app
        .put_json(&format!("/api/v1/watering-plans/{}", plan_id), &finish)
        .await;
    assert_eq!(finish_resp.status().as_u16(), 200);

    // A later aggregate save (any write path touching the plan) must not
    // reset the recorded consumption to the column default.
    let repo = PgWateringPlanRepository::new(app.db_pool.clone());
    let id = domain::Id::new(plan_id.parse().unwrap());
    let aggregate = repo.by_id(id).await.unwrap();
    repo.save(&aggregate).await.unwrap();

    let evals = repo.evaluations(id).await.unwrap();
    assert_eq!(evals.len(), 1);
    assert!(
        (evals[0].consumed_water - 42.5).abs() < 1e-9,
        "consumed_water must survive a re-save, got {}",
        evals[0].consumed_water
    );
}

#[tokio::test]
async fn watering_plan_user_ids_round_trip() {
    let app = spawn_app().await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let user_id = uuid::Uuid::now_v7().to_string();

    let mut body = plan_body(tid, vec![]);
    body["user_ids"] = serde_json::json!([user_id]);
    let response = app.post_json("/api/v1/watering-plans", &body).await;
    assert_eq!(response.status().as_u16(), 201);
    let plan: serde_json::Value = response.json().await.unwrap();
    assert_eq!(plan["user_ids"], serde_json::json!([user_id]));

    let list: serde_json::Value = app
        .get("/api/v1/watering-plans")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(list["data"][0]["user_ids"], serde_json::json!([user_id]));

    let other_user = uuid::Uuid::now_v7().to_string();
    let update = serde_json::json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Bewaesserung Innenstadt",
        "status": "planned",
        "transporter_id": tid,
        "tree_cluster_ids": [],
        "user_ids": [other_user],
        "cancellation_note": ""
    });
    let plan_id = plan["id"].as_str().unwrap();
    let updated: serde_json::Value = app
        .put_json(&format!("/api/v1/watering-plans/{plan_id}"), &update)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(updated["user_ids"], serde_json::json!([other_user]));
}

#[tokio::test]
async fn list_watering_plans_filters_by_status() {
    let app = spawn_app().await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let planned: serde_json::Value = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await
        .json()
        .await
        .unwrap();
    let active: serde_json::Value = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await
        .json()
        .await
        .unwrap();

    let start = serde_json::json!({
        "date": "2026-05-01T08:00:00Z",
        "description": "Bewaesserung Innenstadt",
        "status": "active",
        "transporter_id": tid,
        "tree_cluster_ids": [],
        "user_ids": [],
        "cancellation_note": ""
    });
    let active_id = active["id"].as_str().unwrap();
    let resp = app
        .put_json(&format!("/api/v1/watering-plans/{active_id}"), &start)
        .await;
    assert_eq!(resp.status().as_u16(), 200);

    let only_active: serde_json::Value = app
        .get("/api/v1/watering-plans?status=active")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(only_active["data"].as_array().unwrap().len(), 1);
    assert_eq!(only_active["data"][0]["id"], active["id"]);

    let both: serde_json::Value = app
        .get("/api/v1/watering-plans?status=active&status=planned")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(both["data"].as_array().unwrap().len(), 2);

    let unfiltered: serde_json::Value = app
        .get("/api/v1/watering-plans")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(unfiltered["data"].as_array().unwrap().len(), 2);
    let _ = planned;
}

#[tokio::test]
async fn create_watering_plan_with_invalid_user_id_returns_400() {
    let app = spawn_app().await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();

    let mut body = plan_body(tid, vec![]);
    body["user_ids"] = serde_json::json!(["kein-uuid"]);
    let response = app.post_json("/api/v1/watering-plans", &body).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn route_geometry_round_trips_through_repository() {
    use domain::shared::{coordinates::Coordinate, distance::Distance};
    use domain::watering_plan::{WateringPlanReader, WateringPlanWriter};
    use server::infra::pg_watering_plan::PgWateringPlanRepository;

    let app = spawn_app().await;
    let transporter = create_transporter(&app).await;
    let tid = transporter["id"].as_str().unwrap();
    let resp = app
        .post_json("/api/v1/watering-plans", &plan_body(tid, vec![]))
        .await;
    let plan_json: serde_json::Value = resp.json().await.unwrap();
    let plan_id: uuid::Uuid = plan_json["id"].as_str().unwrap().parse().unwrap();

    let repo = PgWateringPlanRepository::new(app.db_pool.clone());
    let mut plan = repo.by_id(domain::Id::new(plan_id)).await.unwrap();
    let geometry = vec![
        Coordinate::new(54.76, 9.43).unwrap(),
        Coordinate::new(54.80, 9.44).unwrap(),
    ];
    plan.set_metrics(
        Some(Distance::new(1234.0).unwrap()),
        Some(160.0),
        1,
        std::time::Duration::from_secs(900),
        None,
        Some(geometry.clone()),
    );
    plan.start_point_name = Some("Depot Nord".to_string());
    repo.save(&plan).await.unwrap();

    let reloaded = repo.by_id(domain::Id::new(plan_id)).await.unwrap();
    assert_eq!(reloaded.route_geometry(), Some(geometry.as_slice()));
    assert_eq!(reloaded.distance.map(|d| d.meters()), Some(1234.0));
    assert_eq!(reloaded.start_point_name.as_deref(), Some("Depot Nord"));
}
