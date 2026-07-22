use crate::helpers::spawn_app;

pub const ROOT_ORG_ID: &str = "01980000-0000-7000-8000-000000000001";

#[tokio::test]
async fn create_organization_copies_the_five_templates() {
    let app = spawn_app().await;
    let resp = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "TBZ Flensburg", "parent_id": ROOT_ORG_ID }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let org: serde_json::Value = resp.json().await.unwrap();
    let org_id = org["id"].as_str().unwrap();

    let roles: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{org_id}/roles"))
        .await
        .json()
        .await
        .unwrap();
    let names: Vec<&str> = roles
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(roles.as_array().unwrap().len(), 5);
    assert!(names.contains(&"Administrator"));
}

#[tokio::test]
async fn sibling_name_conflict_returns_409() {
    let app = spawn_app().await;
    let body = serde_json::json!({ "name": "TBZ", "parent_id": ROOT_ORG_ID });
    assert_eq!(
        app.post_json("/api/v1/organizations", &body).await.status(),
        201
    );
    assert_eq!(
        app.post_json("/api/v1/organizations", &body).await.status(),
        409
    );
}

#[tokio::test]
async fn list_contains_seeded_root() {
    let app = spawn_app().await;
    let resp = app.get("/api/v1/organizations").await;
    assert_eq!(resp.status(), 200);
    let orgs: serde_json::Value = resp.json().await.unwrap();
    assert!(
        orgs.as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == ROOT_ORG_ID && o["parent_id"].is_null())
    );
}

#[tokio::test]
async fn rename_updates_the_name() {
    let app = spawn_app().await;
    let org: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Alt", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let id = org["id"].as_str().unwrap();
    let resp = app
        .patch_json(
            &format!("/api/v1/organizations/{id}"),
            &serde_json::json!({ "name": "Neu" }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["name"],
        "Neu"
    );
}

#[tokio::test]
async fn delete_with_children_returns_409_and_root_is_immutable() {
    let app = spawn_app().await;
    let parent: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "P", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let pid = parent["id"].as_str().unwrap();
    app.post_json(
        "/api/v1/organizations",
        &serde_json::json!({ "name": "C", "parent_id": pid }),
    )
    .await;

    assert_eq!(
        app.delete(&format!("/api/v1/organizations/{pid}"))
            .await
            .status(),
        409
    );
    assert_eq!(
        app.delete(&format!("/api/v1/organizations/{ROOT_ORG_ID}"))
            .await
            .status(),
        409
    );
}

#[tokio::test]
async fn delete_empty_org_succeeds() {
    let app = spawn_app().await;
    let org: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Weg", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let id = org["id"].as_str().unwrap();
    assert_eq!(
        app.delete(&format!("/api/v1/organizations/{id}"))
            .await
            .status(),
        204
    );
}
