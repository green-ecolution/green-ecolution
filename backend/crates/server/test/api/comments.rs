use serde_json::json;
use uuid::Uuid;

use crate::auth_helpers::spawn_with_auth;
use crate::helpers::{TestApp, seed_user_with_permissions};

async fn create_cluster(app: &TestApp, token: &str, org: Uuid) -> String {
    let created: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/api/v1/clusters", app.address))
        .bearer_auth(token)
        .json(&json!({
            "name": "Kommentar-Gruppe",
            "address": "Am Testfeld 1",
            "description": "",
            "soil_condition": "Su3",
            "tree_ids": [],
            "organization_id": org,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    created["id"].as_str().unwrap().to_owned()
}

const PLAN_PERMS: &[&str] = &[
    "watering_plan:read",
    "watering_plan:create",
    "watering_plan:update",
    "vehicle:read",
    "vehicle:create",
];

fn transporter_payload() -> serde_json::Value {
    json!({
        "number_plate": format!("FL-GE {}", &Uuid::new_v4().to_string()[..8]),
        "description": "Giesswagen",
        "water_capacity": 5000.0,
        "model": "MAN TGS",
        "status": "available",
        "type": "transporter",
        "driving_license": "C",
        "height": 3.2, "width": 2.5, "length": 8.0, "weight": 12000.0
    })
}

/// Creates a transporter and a plan owned by `org`, and returns the plan id.
/// `tree_cluster_ids` stays empty — the comment tests do not need a cluster.
async fn create_plan(app: &TestApp, token: &str, org: Uuid) -> String {
    let client = reqwest::Client::new();
    let transporter: serde_json::Value = client
        .post(format!("{}/api/v1/vehicles", app.address))
        .bearer_auth(token)
        .json(&transporter_payload())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let transporter_id = transporter["id"].as_str().unwrap();

    let created: serde_json::Value = client
        .post(format!("{}/api/v1/watering-plans", app.address))
        .bearer_auth(token)
        .json(&json!({
            "date": "2026-09-15T08:00:00Z",
            "description": "Kommentar-Plan",
            "transporter_id": transporter_id,
            "tree_cluster_ids": [],
            "user_ids": [],
            "organization_id": org,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    created["id"].as_str().unwrap().to_owned()
}

/// `author_name` is deliberately not asserted here: resolving it needs a
/// mocked Keycloak identity for the acting user, and `seed_user_with_permissions`
/// does not hand back the user id. The name is best-effort by design (see
/// `resolve_author_names`), so its absence must never fail a request.
#[tokio::test]
async fn creates_and_lists_a_cluster_comment() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Kommentar Org",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;
    let client = reqwest::Client::new();

    let created = client
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "body": "  Boden war noch feucht  " }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let created: serde_json::Value = created.json().await.unwrap();
    assert_eq!(created["body"], "Boden war noch feucht");
    assert!(created["created_at"].as_str().is_some());

    let list: serde_json::Value = client
        .get(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(list["pagination"]["total_records"], 1);
    assert_eq!(list["data"][0]["body"], "Boden war noch feucht");
    assert_eq!(list["data"][0]["id"], created["id"]);
    assert_eq!(list["data"][0]["author_id"], created["author_id"]);
}

#[tokio::test]
async fn creates_and_lists_a_watering_plan_comment() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(&harness, &app, "Plan Org", PLAN_PERMS).await;
    let plan_id = create_plan(&app, &token, org).await;
    let client = reqwest::Client::new();

    let created = client
        .post(format!(
            "{}/api/v1/watering-plans/{plan_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "body": "Tank war nur halb voll" }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);

    let list: serde_json::Value = client
        .get(format!(
            "{}/api/v1/watering-plans/{plan_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["pagination"]["total_records"], 1);
    assert_eq!(list["data"][0]["body"], "Tank war nur halb voll");
}

#[tokio::test]
async fn cluster_comments_are_not_visible_on_another_cluster() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Zwei Gruppen",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let first = create_cluster(&app, &token, org).await;
    let second = create_cluster(&app, &token, org).await;
    let client = reqwest::Client::new();

    let posted = client
        .post(format!("{}/api/v1/clusters/{first}/comments", app.address))
        .bearer_auth(&token)
        .json(&json!({ "body": "nur hier" }))
        .send()
        .await
        .unwrap();
    assert_eq!(posted.status(), 201);

    let list: serde_json::Value = client
        .get(format!("{}/api/v1/clusters/{second}/comments", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["pagination"]["total_records"], 0);
}

/// `ValidationError` (which `CommentBody::new` returns on an empty/whitespace
/// body) maps to 400 everywhere in this codebase (see
/// `http/v1/error.rs::validation_tests`), not 422 — there is nothing
/// comment-specific about that mapping.
#[tokio::test]
async fn rejects_empty_body_with_400() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Leerer Text",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;

    let response = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "body": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["error"].as_str().is_some(),
        "error bodies must be JSON"
    );
}

#[tokio::test]
async fn create_requires_update_permission_on_the_parent() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Nur Lesen",
        &["tree_cluster:read", "tree_cluster:create"],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;

    let response = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "body": "darf ich nicht" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn foreign_org_gets_404_on_list_and_create() {
    let (harness, app) = spawn_with_auth().await;
    let (org_a, token_a) = seed_user_with_permissions(
        &harness,
        &app,
        "Org A",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let (_org_b, token_b) = seed_user_with_permissions(
        &harness,
        &app,
        "Org B",
        &["tree_cluster:read", "tree_cluster:update"],
    )
    .await;
    let cluster_id = create_cluster(&app, &token_a, org_a).await;
    let client = reqwest::Client::new();

    let list = client
        .get(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(list.status(), 404);

    let create = client
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token_b)
        .json(&json!({ "body": "fremd" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 404);
}

#[tokio::test]
async fn unknown_parent_gets_404() {
    let (harness, app) = spawn_with_auth().await;
    let (_org, token) =
        seed_user_with_permissions(&harness, &app, "Unbekannt", &["tree_cluster:read"]).await;
    let missing = Uuid::now_v7();

    let response = reqwest::Client::new()
        .get(format!(
            "{}/api/v1/clusters/{missing}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn list_is_newest_first_and_paginated() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Sortierung",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;
    let client = reqwest::Client::new();

    for body in ["eins", "zwei", "drei"] {
        client
            .post(format!(
                "{}/api/v1/clusters/{cluster_id}/comments",
                app.address
            ))
            .bearer_auth(&token)
            .json(&json!({ "body": body }))
            .send()
            .await
            .unwrap();
    }

    let first_page: serde_json::Value = client
        .get(format!(
            "{}/api/v1/clusters/{cluster_id}/comments?page=1&per_page=2",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first_page["pagination"]["total_records"], 3);
    assert_eq!(first_page["data"].as_array().unwrap().len(), 2);
    assert_eq!(first_page["data"][0]["body"], "drei");
    assert_eq!(first_page["data"][1]["body"], "zwei");

    let second_page: serde_json::Value = client
        .get(format!(
            "{}/api/v1/clusters/{cluster_id}/comments?page=2&per_page=2",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(second_page["data"].as_array().unwrap().len(), 1);
    assert_eq!(second_page["data"][0]["body"], "eins");
}

/// Seeds a second user in an existing org with the given permissions and
/// returns their token. Used to test that a non-author needs `:delete`.
async fn seed_second_user_in_org(
    harness: &crate::auth_helpers::AuthHarness,
    app: &TestApp,
    org: Uuid,
    role_name: &str,
    permissions: &[&str],
) -> String {
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, $2, $3)
           RETURNING id"#,
        org,
        role_name,
        &permissions,
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)"#,
        user_id,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    harness.sign_token(json!({ "sub": user_id.to_string() }))
}

async fn post_comment(app: &TestApp, token: &str, cluster_id: &str, body: &str) -> String {
    let created: serde_json::Value = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(token)
        .json(&json!({ "body": body }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    created["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn author_deletes_own_comment_without_delete_permission() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Autor löscht",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;
    let comment_id = post_comment(&app, &token, &cluster_id, "weg damit").await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "{}/api/v1/clusters/{cluster_id}/comments/{comment_id}",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);

    let list: serde_json::Value = client
        .get(format!(
            "{}/api/v1/clusters/{cluster_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn deleting_twice_gets_404() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Doppelt löschen",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &token, org).await;
    let comment_id = post_comment(&app, &token, &cluster_id, "einmalig").await;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/v1/clusters/{cluster_id}/comments/{comment_id}",
        app.address
    );

    assert_eq!(
        client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}

#[tokio::test]
async fn non_author_needs_delete_permission() {
    let (harness, app) = spawn_with_auth().await;
    let (org, author_token) = seed_user_with_permissions(
        &harness,
        &app,
        "Fremdlöschung",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let cluster_id = create_cluster(&app, &author_token, org).await;
    let comment_id = post_comment(&app, &author_token, &cluster_id, "nicht deiner").await;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/v1/clusters/{cluster_id}/comments/{comment_id}",
        app.address
    );

    let reader_token =
        seed_second_user_in_org(&harness, &app, org, "Reader", &["tree_cluster:read"]).await;
    assert_eq!(
        client
            .delete(&url)
            .bearer_auth(&reader_token)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );

    let moderator_token = seed_second_user_in_org(
        &harness,
        &app,
        org,
        "Moderator",
        &["tree_cluster:read", "tree_cluster:delete"],
    )
    .await;
    assert_eq!(
        client
            .delete(&url)
            .bearer_auth(&moderator_token)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
}

#[tokio::test]
async fn comment_id_of_another_parent_gets_404() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(
        &harness,
        &app,
        "Falscher Elternpfad",
        &[
            "tree_cluster:read",
            "tree_cluster:create",
            "tree_cluster:update",
        ],
    )
    .await;
    let first = create_cluster(&app, &token, org).await;
    let second = create_cluster(&app, &token, org).await;
    let comment_id = post_comment(&app, &token, &first, "gehört zu first").await;

    let response = reqwest::Client::new()
        .delete(format!(
            "{}/api/v1/clusters/{second}/comments/{comment_id}",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn plan_comment_can_be_deleted_by_its_author() {
    let (harness, app) = spawn_with_auth().await;
    let (org, token) = seed_user_with_permissions(&harness, &app, "Plan löschen", PLAN_PERMS).await;
    let plan_id = create_plan(&app, &token, org).await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(format!(
            "{}/api/v1/watering-plans/{plan_id}/comments",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "body": "Plan-Notiz" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let comment_id = created["id"].as_str().unwrap();

    let response = client
        .delete(format!(
            "{}/api/v1/watering-plans/{plan_id}/comments/{comment_id}",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
}
