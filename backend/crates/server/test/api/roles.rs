use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

async fn create_org(app: &crate::helpers::TestApp, name: &str) -> String {
    let resp = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": name, "parent_id": ROOT_ORG_ID }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn templates_endpoint_lists_the_five_seeded_templates() {
    let app = spawn_app().await;
    let templates: serde_json::Value = app
        .get("/api/v1/roles/templates")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(templates.as_array().unwrap().len(), 5);
    assert!(
        templates
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["is_template"] == true)
    );
}

#[tokio::test]
async fn permissions_endpoint_lists_the_catalog() {
    let app = spawn_app().await;
    let perms: serde_json::Value = app.get("/api/v1/permissions").await.json().await.unwrap();
    assert_eq!(perms.as_array().unwrap().len(), 36);
    assert!(
        perms
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("tree:read"))
    );
}

#[tokio::test]
async fn create_role_from_scratch_and_duplicate_name_conflicts() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let body =
        serde_json::json!({ "name": "Gießtrupp", "permissions": ["tree:read", "tree:update"] });
    let resp = app
        .post_json(&format!("/api/v1/organizations/{org}/roles"), &body)
        .await;
    assert_eq!(resp.status(), 201);
    assert_eq!(
        app.post_json(&format!("/api/v1/organizations/{org}/roles"), &body)
            .await
            .status(),
        409
    );
}

#[tokio::test]
async fn create_role_rejects_unknown_permission() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let body = serde_json::json!({ "name": "Kaputt", "permissions": ["garden:fly"] });
    assert_eq!(
        app.post_json(&format!("/api/v1/organizations/{org}/roles"), &body)
            .await
            .status(),
        400
    );
}

#[tokio::test]
async fn copy_role_binds_copy_to_target_org() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let templates: serde_json::Value = app
        .get("/api/v1/roles/templates")
        .await
        .json()
        .await
        .unwrap();
    let template_id = templates.as_array().unwrap()[0]["id"].as_str().unwrap();
    // Kopie unter neuem Namen nötig, weil die Template-Kopien beim Org-Anlegen
    // die Originalnamen bereits belegen.
    let resp = app
        .post_json(
            &format!("/api/v1/organizations/{org}/roles"),
            &serde_json::json!({ "copy_from_role_id": template_id, "name": "Admin Kopie" }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let role: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(role["organization_id"], org.as_str());
    assert_eq!(role["is_template"], false);
}

#[tokio::test]
async fn update_and_delete_work_for_org_roles_but_not_templates() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let role: serde_json::Value = app
        .post_json(
            &format!("/api/v1/organizations/{org}/roles"),
            &serde_json::json!({ "name": "Temp", "permissions": ["tree:read"] }),
        )
        .await
        .json()
        .await
        .unwrap();
    let role_id = role["id"].as_str().unwrap();

    let resp = app
        .patch_json(
            &format!("/api/v1/roles/{role_id}"),
            &serde_json::json!({ "name": "Umbenannt", "description": "Neu", "permissions": ["tree:read", "sensor:read"] }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["permissions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    assert_eq!(
        app.delete(&format!("/api/v1/roles/{role_id}"))
            .await
            .status(),
        204
    );

    let templates: serde_json::Value = app
        .get("/api/v1/roles/templates")
        .await
        .json()
        .await
        .unwrap();
    let template_id = templates.as_array().unwrap()[0]["id"].as_str().unwrap();
    assert_eq!(
        app.patch_json(
            &format!("/api/v1/roles/{template_id}"),
            &serde_json::json!({ "name": "X", "description": null, "permissions": [] })
        )
        .await
        .status(),
        409
    );
    assert_eq!(
        app.delete(&format!("/api/v1/roles/{template_id}"))
            .await
            .status(),
        409
    );
}
