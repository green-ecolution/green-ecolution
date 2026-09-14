use serde_json::json;
use uuid::Uuid;

use crate::helpers::{install_plugin, spawn_app_with_plugins};

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

fn item(external_id: &str, species: &str) -> serde_json::Value {
    json!({
        "external_id": external_id,
        "number": "FL-001",
        "species": species,
        "planting_year": 1998,
        "latitude": 54.7836,
        "longitude": 9.4321,
        "description": null,
        "additional_info": { "objectid": 12345 }
    })
}

#[tokio::test]
async fn first_run_creates_a_tree_with_provenance() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(
        &app,
        "kataster",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;

    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": [item("1", "Quercus robur")] }),
            &key,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["results"][0]["status"], "created");
    assert_eq!(body["summary"]["created"], 1);

    let tree_id = body["results"][0]["tree_id"].as_str().unwrap();
    let provider: Option<String> = sqlx::query_scalar!(
        "SELECT provider FROM trees WHERE id = $1",
        uuid::Uuid::parse_str(tree_id).unwrap()
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(provider.as_deref(), Some("kataster"));
}

#[tokio::test]
async fn second_run_updates_and_third_run_is_unchanged() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(
        &app,
        "kataster",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;
    let path = "/api/v1/plugins/ingest/trees";

    app.post_json_with_bearer(
        path,
        &json!({ "items": [item("1", "Quercus robur")] }),
        &key,
    )
    .await;

    let updated: serde_json::Value = app
        .post_json_with_bearer(
            path,
            &json!({ "items": [item("1", "Tilia cordata")] }),
            &key,
        )
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(updated["results"][0]["status"], "updated");

    let again: serde_json::Value = app
        .post_json_with_bearer(
            path,
            &json!({ "items": [item("1", "Tilia cordata")] }),
            &key,
        )
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(again["results"][0]["status"], "unchanged");
}

#[tokio::test]
async fn a_broken_item_does_not_fail_the_batch() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(
        &app,
        "kataster",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;
    let mut broken = item("2", "");
    broken["species"] = json!("");

    let body: serde_json::Value = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": [item("1", "Quercus robur"), broken] }),
            &key,
        )
        .await
        .json()
        .await
        .unwrap();

    assert_eq!(body["summary"]["created"], 1);
    assert_eq!(body["summary"]["failed"], 1);
    assert!(
        body["results"][1]["error"]
            .as_str()
            .unwrap()
            .contains("species")
    );
}

#[tokio::test]
async fn missing_tree_create_permission_is_forbidden() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(&app, "readonly", &["tree:read"]).await;

    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": [item("1", "Quercus robur")] }),
            &key,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[tokio::test]
async fn oversized_batch_is_rejected() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(
        &app,
        "kataster",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;
    let items: Vec<serde_json::Value> = (0..501)
        .map(|i| item(&i.to_string(), "Quercus robur"))
        .collect();

    let resp = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": items }),
            &key,
        )
        .await;
    assert_eq!(resp.status().as_u16(), 413);
}

#[tokio::test]
async fn delete_removes_tree_and_ref() {
    let app = spawn_app_with_plugins().await;
    let key = install_plugin(
        &app,
        "kataster",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;
    app.post_json_with_bearer(
        "/api/v1/plugins/ingest/trees",
        &json!({ "items": [item("1", "Quercus robur")] }),
        &key,
    )
    .await;

    let resp = app
        .delete_with_bearer("/api/v1/plugins/ingest/trees/1", &key)
        .await;
    assert_eq!(resp.status().as_u16(), 204);

    let left: i64 = sqlx::query_scalar!("SELECT count(*) FROM plugin_tree_refs")
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
        .unwrap_or(0);
    assert_eq!(left, 0);

    assert_eq!(
        app.delete_with_bearer("/api/v1/plugins/ingest/trees/1", &key)
            .await
            .status()
            .as_u16(),
        404
    );
}

/// Critical regression: a tree the plugin created can be moved to a
/// different organization afterwards (a clusterless tree has no other lock
/// on it), and the reference in `plugin_tree_refs` survives that move
/// untouched. Without checking the tree's *current* organization, the
/// plugin's next import would silently overwrite a tree that no longer
/// belongs to its tenant, and could delete it outright.
#[tokio::test]
async fn transferring_the_tree_revokes_the_plugins_update_and_delete_rights() {
    let app = spawn_app_with_plugins().await;

    // Two siblings directly under root: a plugin scoped to one has no claim
    // over the other through subtree inheritance.
    let org_a = Uuid::new_v4();
    let org_b = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'Org A')",
        org_a,
        Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'Org B')",
        org_b,
        Uuid::parse_str(ROOT_ORG).unwrap(),
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let created_plugin: serde_json::Value = app
        .post_json(
            "/api/v1/plugins",
            &json!({
                "slug": "scoped-kataster",
                "name": "scoped-kataster",
                "organization_id": org_a,
                "permissions": ["tree:read", "tree:create", "tree:update", "tree:delete"],
                "required_permissions": [],
                "frontend": { "mode": "none" }
            }),
        )
        .await
        .json()
        .await
        .unwrap();
    app.patch_json(
        "/api/v1/plugins/scoped-kataster",
        &json!({ "enabled": true }),
    )
    .await;
    let key = created_plugin["key"].as_str().unwrap().to_string();

    let created: serde_json::Value = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": [item("1", "Quercus robur")] }),
            &key,
        )
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(created["results"][0]["status"], "created");
    let tree_id = created["results"][0]["tree_id"].as_str().unwrap();

    let transfer_resp = app
        .patch_json(
            &format!("/api/v1/trees/{tree_id}/organization"),
            &json!({ "organization_id": org_b }),
        )
        .await;
    assert_eq!(transfer_resp.status().as_u16(), 204);

    let updated: serde_json::Value = app
        .post_json_with_bearer(
            "/api/v1/plugins/ingest/trees",
            &json!({ "items": [item("1", "Tilia cordata")] }),
            &key,
        )
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(updated["results"][0]["status"], "failed");

    let delete_resp = app
        .delete_with_bearer("/api/v1/plugins/ingest/trees/1", &key)
        .await;
    assert_eq!(delete_resp.status().as_u16(), 403);

    let species: String = sqlx::query_scalar!(
        "SELECT species FROM trees WHERE id = $1",
        Uuid::parse_str(tree_id).unwrap()
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(
        species, "Quercus robur",
        "the transferred tree must be untouched"
    );
}

#[tokio::test]
async fn refs_are_listed_per_plugin_and_paged() {
    let app = spawn_app_with_plugins().await;
    let mine = install_plugin(
        &app,
        "mine",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;
    let other = install_plugin(
        &app,
        "other",
        &["tree:read", "tree:create", "tree:update", "tree:delete"],
    )
    .await;

    let items: Vec<serde_json::Value> = (0..3)
        .map(|i| item(&format!("ext-{i}"), "Quercus robur"))
        .collect();
    app.post_json_with_bearer(
        "/api/v1/plugins/ingest/trees",
        &json!({ "items": items }),
        &mine,
    )
    .await;
    app.post_json_with_bearer(
        "/api/v1/plugins/ingest/trees",
        &json!({ "items": [item("foreign", "Tilia")] }),
        &other,
    )
    .await;

    let page: serde_json::Value = app
        .get_with_bearer("/api/v1/plugins/ingest/trees?limit=2", &mine)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["external_id"], "ext-0");

    let next: serde_json::Value = app
        .get_with_bearer("/api/v1/plugins/ingest/trees?limit=2&cursor=ext-1", &mine)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(next["items"].as_array().unwrap().len(), 1);

    let all_mine: serde_json::Value = app
        .get_with_bearer("/api/v1/plugins/ingest/trees?limit=500", &mine)
        .await
        .json()
        .await
        .unwrap();
    assert!(
        !all_mine["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["external_id"] == "foreign"),
        "a plugin must never see another plugin's refs"
    );
}
