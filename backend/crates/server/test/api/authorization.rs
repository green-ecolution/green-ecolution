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

/// Signs a token for a brand-new subject with no user_profiles row and no
/// role_assignments — i.e. zero grants anywhere.
fn zero_grant_token(harness: &crate::auth_helpers::AuthHarness) -> String {
    harness.sign_token(json!({ "sub": Uuid::new_v4().to_string() }))
}

/// Seeds an org (child of root) and an org-owned role within it (not a
/// template, since templates 409 before authz is ever reached).
async fn seed_org_and_role(
    app: &crate::helpers::TestApp,
    org_name: &str,
    role_name: &str,
) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1::uuid, $2) RETURNING id"#,
        Uuid::parse_str(ROOT_ORG_ID).unwrap(),
        org_name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, $2, ARRAY['tree:read'])
           RETURNING id"#,
        org_id,
        role_name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    (org_id, role_id)
}

#[tokio::test]
async fn organization_mutations_are_forbidden_without_grants() {
    let (harness, app) = spawn_with_auth().await;
    let (org_id, _role_id) = seed_org_and_role(&app, "Zero-Grant-Org", "Zero-Grant-Rolle").await;
    let token = zero_grant_token(&harness);

    let resp = reqwest::Client::new()
        .patch(format!("{}/api/v1/organizations/{org_id}", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "Umbenannt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = reqwest::Client::new()
        .delete(format!("{}/api/v1/organizations/{org_id}", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn role_mutations_are_forbidden_without_grants() {
    let (harness, app) = spawn_with_auth().await;
    let (_org_id, role_id) = seed_org_and_role(&app, "Zero-Grant-Org-Rollen", "Org-Rolle").await;
    let token = zero_grant_token(&harness);

    let resp = reqwest::Client::new()
        .patch(format!("{}/api/v1/roles/{role_id}", app.address))
        .bearer_auth(&token)
        .json(&json!({ "name": "X", "description": null, "permissions": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = reqwest::Client::new()
        .delete(format!("{}/api/v1/roles/{role_id}", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn user_role_assignment_is_forbidden_without_grants() {
    let (harness, app) = spawn_with_auth().await;
    let (_org_id, role_id) = seed_org_and_role(&app, "Zero-Grant-Org-Assign", "Assign-Rolle").await;
    let token = zero_grant_token(&harness);
    let target_user_id = Uuid::new_v4();

    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/users/{target_user_id}/roles",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "role_id": role_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = reqwest::Client::new()
        .delete(format!(
            "{}/api/v1/users/{target_user_id}/roles/{role_id}",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn user_organization_change_is_forbidden_without_grants() {
    let (harness, app) = spawn_with_auth().await;
    let (org_id, _role_id) = seed_org_and_role(&app, "Zero-Grant-Org-Move", "Move-Rolle").await;
    let token = zero_grant_token(&harness);
    let target_user_id = Uuid::new_v4();

    let resp = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/users/{target_user_id}/organization",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "organization_id": org_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
