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

/// Real-auth harness for the self-lockout guard (OP#3133). The demo bypass
/// grants unrestricted access, so these cases only exist with `auth.enabled`.
mod self_lockout {
    use crate::{auth_helpers::spawn_with_auth, organizations::ROOT_ORG_ID};
    use serde_json::json;
    use uuid::Uuid;

    /// Seeds an org under root, a role carrying `permissions`, and a user in
    /// that org holding it. Returns (org_id, role_id, token).
    async fn seed_user_holding(
        harness: &crate::auth_helpers::AuthHarness,
        app: &crate::helpers::TestApp,
        org_name: &str,
        permissions: &[&str],
    ) -> (Uuid, Uuid, String) {
        let org_id: Uuid = sqlx::query_scalar!(
            r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
            Uuid::parse_str(ROOT_ORG_ID).unwrap(),
            org_name
        )
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
        let role_id = insert_role(app, org_id, "Org-Admin", permissions).await;
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
            user_id,
            org_id
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
        let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
        (org_id, role_id, token)
    }

    async fn insert_role(
        app: &crate::helpers::TestApp,
        org_id: Uuid,
        name: &str,
        permissions: &[&str],
    ) -> Uuid {
        let permissions: Vec<String> = permissions.iter().map(|p| (*p).to_string()).collect();
        sqlx::query_scalar!(
            r#"INSERT INTO roles (id, organization_id, name, permissions)
               VALUES (gen_random_uuid(), $1, $2, $3) RETURNING id"#,
            org_id,
            name,
            &permissions
        )
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
    }

    async fn permissions_of(app: &crate::helpers::TestApp, role_id: Uuid) -> Vec<String> {
        sqlx::query_scalar!(r#"SELECT permissions FROM roles WHERE id = $1"#, role_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap()
    }

    async fn patch_role(
        app: &crate::helpers::TestApp,
        token: &str,
        role_id: Uuid,
        name: &str,
        permissions: &[&str],
    ) -> reqwest::Response {
        reqwest::Client::new()
            .patch(format!("{}/api/v1/roles/{role_id}", app.address))
            .bearer_auth(token)
            .json(&json!({ "name": name, "description": null, "permissions": permissions }))
            .send()
            .await
            .unwrap()
    }

    const ADMIN: [&str; 4] = ["role:update", "role:delete", "user:update", "tree:read"];

    #[tokio::test]
    async fn emptying_your_own_administration_role_returns_409() {
        let (harness, app) = spawn_with_auth().await;
        let (_org, role_id, token) = seed_user_holding(&harness, &app, "TBZ", &ADMIN).await;

        let resp = patch_role(&app, &token, role_id, "Org-Admin", &["tree:read"]).await;

        assert_eq!(resp.status(), 409);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(
            body["error"],
            "this change would remove your own right to administer roles and members"
        );

        // Side-effect assertion: the role must be untouched.
        let after = permissions_of(&app, role_id).await;
        assert!(after.contains(&"role:update".to_string()));
        assert!(after.contains(&"user:update".to_string()));
    }

    #[tokio::test]
    async fn deleting_your_own_administration_role_returns_409() {
        let (harness, app) = spawn_with_auth().await;
        let (_org, role_id, token) = seed_user_holding(&harness, &app, "TBZ", &ADMIN).await;

        let resp = reqwest::Client::new()
            .delete(format!("{}/api/v1/roles/{role_id}", app.address))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 409);

        // Side-effect assertion: the role must still exist.
        let still_there: Option<i64> =
            sqlx::query_scalar!(r#"SELECT COUNT(*) FROM roles WHERE id = $1"#, role_id)
                .fetch_one(&app.db_pool)
                .await
                .unwrap();
        assert_eq!(still_there, Some(1));
    }

    #[tokio::test]
    async fn maintaining_your_own_role_stays_allowed() {
        let (harness, app) = spawn_with_auth().await;
        let (_org, role_id, token) = seed_user_holding(&harness, &app, "TBZ", &ADMIN).await;

        // Renamed and stripped of an unrelated permission, but still
        // administering: the guard must not block this.
        let resp = patch_role(
            &app,
            &token,
            role_id,
            "Verwaltung Nord",
            &["role:update", "user:update"],
        )
        .await;

        assert_eq!(resp.status(), 200);
        assert_eq!(permissions_of(&app, role_id).await.len(), 2);
    }

    #[tokio::test]
    async fn editing_a_role_you_do_not_hold_stays_allowed() {
        let (harness, app) = spawn_with_auth().await;
        let (org, _own_role, token) = seed_user_holding(&harness, &app, "TBZ", &ADMIN).await;
        let other = insert_role(&app, org, "Gießtrupp", &["tree:read"]).await;

        let resp = patch_role(&app, &token, other, "Gießtrupp", &[]).await;

        assert_eq!(resp.status(), 200);
        assert!(permissions_of(&app, other).await.is_empty());
    }

    #[tokio::test]
    async fn missing_role_update_still_returns_403_not_409() {
        let (harness, app) = spawn_with_auth().await;
        // No role:update at all, so the authorization check must fire before
        // the self-lockout guard ever runs.
        let (_org, role_id, token) =
            seed_user_holding(&harness, &app, "TBZ", &["role:read", "user:update"]).await;

        let resp = patch_role(&app, &token, role_id, "Org-Admin", &[]).await;

        assert_eq!(resp.status(), 403);
    }
}
