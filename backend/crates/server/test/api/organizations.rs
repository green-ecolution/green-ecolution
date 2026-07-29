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
        .put_json(
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

#[tokio::test]
async fn update_sets_address_and_returns_it() {
    let app = spawn_app().await;
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Stadtgärtnerei Nord", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = created["id"].as_str().unwrap().to_owned();

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org_id}"),
            &serde_json::json!({
                "name": "Stadtgärtnerei Nord",
                "address": {
                    "street": "Nordergraben 12",
                    "postal_code": "24937",
                    "city": "Flensburg"
                },
                "contact_person_id": null
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let detail: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{org_id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(detail["address"]["postal_code"], "24937");
    assert_eq!(detail["address"]["city"], "Flensburg");
    assert!(detail["contact_person"].is_null());
}

#[tokio::test]
async fn update_clears_the_address_when_omitted() {
    let app = spawn_app().await;
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Bauhof Mürwik", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = created["id"].as_str().unwrap().to_owned();
    let path = format!("/api/v1/organizations/{org_id}");

    app.put_json(
        &path,
        &serde_json::json!({
            "name": "Bauhof Mürwik",
            "address": { "street": "Am Hafen 3", "postal_code": "24944", "city": "Flensburg" }
        }),
    )
    .await;

    let resp = app
        .put_json(&path, &serde_json::json!({ "name": "Bauhof Mürwik" }))
        .await;
    assert_eq!(resp.status(), 200);

    let detail: serde_json::Value = app.get(&path).await.json().await.unwrap();
    assert!(detail["address"].is_null());
}

#[tokio::test]
async fn update_rejects_a_partial_address() {
    let app = spawn_app().await;
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Team Duburg", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = created["id"].as_str().unwrap();

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org_id}"),
            &serde_json::json!({
                "name": "Team Duburg",
                "address": { "street": "Am Hafen 3", "postal_code": "", "city": "Flensburg" }
            }),
        )
        .await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn update_rejects_a_contact_person_from_another_organization() {
    let app = spawn_app().await;
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Team Jürgensby", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = created["id"].as_str().unwrap();

    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{org_id}"),
            &serde_json::json!({
                "name": "Team Jürgensby",
                "contact_person_id": "01980000-0000-7000-8000-0000000000ff"
            }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn update_rejects_the_root() {
    let app = spawn_app().await;
    let resp = app
        .put_json(
            &format!("/api/v1/organizations/{ROOT_ORG_ID}"),
            &serde_json::json!({ "name": "Anders" }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn list_reports_member_counts() {
    let app = spawn_app().await;
    let orgs: serde_json::Value = app.get("/api/v1/organizations").await.json().await.unwrap();
    let root = orgs
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == ROOT_ORG_ID)
        .unwrap();
    assert!(root["member_count"].is_number());
}

#[tokio::test]
async fn listing_organizations_without_read_permission_is_forbidden() {
    use crate::auth_helpers::spawn_with_auth;
    use crate::helpers::seed_user_with_permissions;

    let (harness, app) = spawn_with_auth().await;
    let (_org, token) =
        seed_user_with_permissions(&harness, &app, "No Org Read", &["tree:read"]).await;

    let status = reqwest::Client::new()
        .get(format!("{}/api/v1/organizations", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, 403);
}
