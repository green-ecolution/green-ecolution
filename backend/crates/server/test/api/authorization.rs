use crate::{auth_helpers::spawn_with_auth, organizations::ROOT_ORG_ID};
use serde_json::json;
use uuid::Uuid;

/// Seeds an org + admin-copy role + a user holding it, returns (org_id, token).
async fn seed_admin_in_new_org(
    harness: &crate::auth_helpers::AuthHarness,
    app: &crate::helpers::TestApp,
    name: &str,
) -> (String, String) {
    let org_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(ROOT_ORG_ID).unwrap(),
        name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           SELECT gen_random_uuid(), $1, 'Org-Admin', permissions FROM roles WHERE id = '01980000-0000-7000-8000-0000000000a1'
           RETURNING id"#,
        org_id
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
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
    (org_id.to_string(), token)
}

#[tokio::test]
async fn role_scope_grants_subtree_but_never_upwards() {
    let (harness, app) = spawn_with_auth().await;
    let (org_id, token) = seed_admin_in_new_org(&harness, &app, "TBZ").await;

    // In the caller's own org (subtree): create a sub-org → 201.
    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/organizations", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "GaLaBau", "parent_id": org_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // At the root (above the caller's org): → 403.
    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/organizations", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "Fremd", "parent_id": ROOT_ORG_ID }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn delegation_cannot_exceed_own_permissions() {
    let (harness, app) = spawn_with_auth().await;

    let org_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(ROOT_ORG_ID).unwrap(),
        "Delegator"
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    // role:create lets the request reach the superset check; tree:delete is
    // deliberately withheld so the caller cannot delegate it.
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Limited', ARRAY['tree:read', 'role:create'])
           RETURNING id"#,
        org_id
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
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

    // Within the caller's own permission set → 201.
    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/organizations/{org_id}/roles",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "name": "Erlaubt", "permissions": ["tree:read"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // Exceeding the caller's own permissions → 403.
    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/organizations/{org_id}/roles",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "name": "Zuviel", "permissions": ["tree:read", "tree:delete"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
