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

/// Creates a zero-grant user and returns both its UUID and a signed token.
fn zero_grant_user(harness: &crate::auth_helpers::AuthHarness) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let token = harness.sign_token(json!({ "sub": user_id.to_string() }));
    (user_id, token)
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
        .put(format!("{}/api/v1/organizations/{org_id}", app.address))
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

async fn seed_child_org(app: &crate::helpers::TestApp, parent: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO organizations (id, parent_id, name) VALUES (gen_random_uuid(), $1, $2) RETURNING id"#,
        parent,
        name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

/// A member of `org` without any role assignment.
async fn seed_member(app: &crate::helpers::TestApp, org: Uuid) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    user_id
}

async fn grant_in(
    app: &crate::helpers::TestApp,
    user: Uuid,
    org: Uuid,
    permissions: &[&str],
) -> Uuid {
    let permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let role_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, 'Scoping-Rolle', $2)
           RETURNING id"#,
        org,
        &permissions
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)"#,
        user,
        role_id
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
    role_id
}

struct UserListFixture {
    caller_token: String,
    caller: Uuid,
    in_own_org: Uuid,
    in_sub_org: Uuid,
    in_parent_org: Uuid,
    in_sibling_org: Uuid,
    sibling_org: Uuid,
}

impl UserListFixture {
    /// Every seeded user, for mocking the IdP side.
    fn everyone(&self) -> Vec<(Uuid, &'static str)> {
        vec![
            (self.caller, "caller"),
            (self.in_own_org, "own"),
            (self.in_sub_org, "sub"),
            (self.in_parent_org, "parent"),
            (self.in_sibling_org, "sibling"),
        ]
    }

    fn visible_ids(&self) -> std::collections::BTreeSet<Uuid> {
        [self.caller, self.in_own_org, self.in_sub_org]
            .into_iter()
            .collect()
    }
}

/// ```text
/// root
/// └── Dach ............... in_parent_org
///     ├── Eigene ........ caller, in_own_org   <- caller holds user:read here
///     │   └── Unter ..... in_sub_org
///     └── Schwester ..... in_sibling_org
/// ```
async fn seed_user_list_tree(
    harness: &crate::auth_helpers::AuthHarness,
    app: &crate::helpers::TestApp,
) -> UserListFixture {
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let parent_org = seed_child_org(app, root, "Dach").await;
    let own_org = seed_child_org(app, parent_org, "Eigene").await;
    let sub_org = seed_child_org(app, own_org, "Unter").await;
    let sibling_org = seed_child_org(app, parent_org, "Schwester").await;

    let caller = seed_member(app, own_org).await;
    grant_in(app, caller, own_org, &["user:read"]).await;

    UserListFixture {
        caller_token: harness.sign_token(json!({ "sub": caller.to_string() })),
        caller,
        in_own_org: seed_member(app, own_org).await,
        in_sub_org: seed_member(app, sub_org).await,
        in_parent_org: seed_member(app, parent_org).await,
        in_sibling_org: seed_member(app, sibling_org).await,
        sibling_org,
    }
}

async fn get_users(app: &crate::helpers::TestApp, token: &str, query: &str) -> reqwest::Response {
    reqwest::Client::new()
        .get(format!("{}/api/v1/users{query}", app.address))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

fn returned_ids(body: &serde_json::Value) -> std::collections::BTreeSet<Uuid> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| Uuid::parse_str(u["id"].as_str().unwrap()).unwrap())
        .collect()
}

#[tokio::test]
async fn user_list_is_forbidden_without_user_read() {
    let (harness, app) = spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let org = seed_child_org(&app, root, "Ohne-User-Read").await;
    let caller = seed_member(&app, org).await;
    grant_in(&app, caller, org, &["tree:read"]).await;
    let token = harness.sign_token(json!({ "sub": caller.to_string() }));

    let resp = get_users(&app, &token, "").await;

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn user_list_shows_the_visible_subtree_only() {
    let (harness, app) = spawn_with_auth().await;
    let fixture = seed_user_list_tree(&harness, &app).await;
    harness.mock_identity_lookups(&fixture.everyone()).await;
    // The unscoped listing endpoint answers with everyone as well, so an
    // unscoped implementation fails on the returned ids instead of erroring
    // out on a missing mock.
    harness.mock_identity_search(&fixture.everyone()).await;

    let resp = get_users(&app, &fixture.caller_token, "?per_page=100").await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(returned_ids(&body), fixture.visible_ids());
    assert_eq!(body["pagination"]["total_records"], 3);
}

#[tokio::test]
async fn user_search_is_intersected_with_the_visible_subtree() {
    let (harness, app) = spawn_with_auth().await;
    let fixture = seed_user_list_tree(&harness, &app).await;
    // The IdP answers with everyone regardless of the query — it knows no
    // organizations, so the scoping must happen on our side.
    harness.mock_identity_search(&fixture.everyone()).await;

    let resp = get_users(&app, &fixture.caller_token, "?query=test&per_page=100").await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(returned_ids(&body), fixture.visible_ids());
    assert_eq!(body["pagination"]["total_records"], 3);
}

#[tokio::test]
async fn user_list_filtered_by_an_invisible_organization_is_empty() {
    let (harness, app) = spawn_with_auth().await;
    let fixture = seed_user_list_tree(&harness, &app).await;
    harness.mock_identity_lookups(&fixture.everyone()).await;

    let resp = get_users(
        &app,
        &fixture.caller_token,
        &format!("?organization_id={}", fixture.sibling_org),
    )
    .await;

    // Nothing visible there rather than a 403: the filter narrows, it never widens.
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(returned_ids(&body).is_empty());
    assert_eq!(body["pagination"]["total_records"], 0);
}

#[tokio::test]
async fn listing_roles_of_a_user_outside_the_caller_scope_is_forbidden() {
    let (harness, app) = spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let caller_org = seed_child_org(&app, root, "Rollen-Lese-Org").await;
    let other_org = seed_child_org(&app, root, "Andere-Org").await;

    let caller = seed_member(&app, caller_org).await;
    grant_in(&app, caller, caller_org, &["user:read"]).await;
    let token = harness.sign_token(json!({ "sub": caller.to_string() }));

    // A sibling org, not a descendant of caller_org — outside the grant's subtree.
    let target = seed_member(&app, other_org).await;

    let resp = reqwest::Client::new()
        .get(format!("{}/api/v1/users/{target}/roles", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn role_assignment_on_self_with_zero_grants_returns_403_not_409() {
    let (harness, app) = spawn_with_auth().await;
    let (_org_id, role_id) = seed_org_and_role(&app, "Self-Assign-Org", "Self-Assign-Role").await;
    let (user_id, token) = zero_grant_user(&harness);

    let resp = reqwest::Client::new()
        .post(format!("{}/api/v1/users/{user_id}/roles", app.address))
        .bearer_auth(&token)
        .json(&json!({ "role_id": role_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "403 (permission denied) must come before 409 (self-target)"
    );
}

#[tokio::test]
async fn role_revocation_on_self_with_zero_grants_returns_403_not_409() {
    let (harness, app) = spawn_with_auth().await;
    let (_org_id, role_id) = seed_org_and_role(&app, "Self-Revoke-Org", "Self-Revoke-Role").await;
    let (user_id, token) = zero_grant_user(&harness);

    let resp = reqwest::Client::new()
        .delete(format!(
            "{}/api/v1/users/{user_id}/roles/{role_id}",
            app.address
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "403 (permission denied) must come before 409 (self-target)"
    );
}

#[tokio::test]
async fn organization_change_on_self_with_zero_grants_returns_403_not_409() {
    let (harness, app) = spawn_with_auth().await;
    let (org_id, _role_id) = seed_org_and_role(&app, "Self-Move-Org", "Self-Move-Role").await;
    let (user_id, token) = zero_grant_user(&harness);

    let resp = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/users/{user_id}/organization",
            app.address
        ))
        .bearer_auth(&token)
        .json(&json!({ "organization_id": org_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        403,
        "403 (permission denied) must come before 409 (self-target)"
    );
}

async fn seed_role(app: &crate::helpers::TestApp, org: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO roles (id, organization_id, name, permissions)
           VALUES (gen_random_uuid(), $1, $2, ARRAY['tree:read'])
           RETURNING id"#,
        org,
        name
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap()
}

struct RoleListFixture {
    caller_token: String,
    // The role `grant_in` creates to carry the caller's own grant; it lives
    // in own_org, so it is itself part of the expected result.
    callers_own_role: Uuid,
    in_own_org: Uuid,
    in_sub_org: Uuid,
    in_parent_org: Uuid,
    in_sibling_org: Uuid,
}

impl RoleListFixture {
    fn visible_ids(&self) -> std::collections::BTreeSet<Uuid> {
        [self.callers_own_role, self.in_own_org, self.in_sub_org]
            .into_iter()
            .collect()
    }
}

/// Same tree shape as `seed_user_list_tree`, but each org owns one role
/// instead of a member.
async fn seed_role_list_tree(
    harness: &crate::auth_helpers::AuthHarness,
    app: &crate::helpers::TestApp,
) -> RoleListFixture {
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let parent_org = seed_child_org(app, root, "Rollen-Dach").await;
    let own_org = seed_child_org(app, parent_org, "Rollen-Eigene").await;
    let sub_org = seed_child_org(app, own_org, "Rollen-Unter").await;
    let sibling_org = seed_child_org(app, parent_org, "Rollen-Schwester").await;

    let caller = seed_member(app, own_org).await;
    let callers_own_role = grant_in(app, caller, own_org, &["role:read"]).await;

    RoleListFixture {
        caller_token: harness.sign_token(json!({ "sub": caller.to_string() })),
        callers_own_role,
        in_own_org: seed_role(app, own_org, "Rolle-Eigene").await,
        in_sub_org: seed_role(app, sub_org, "Rolle-Unter").await,
        in_parent_org: seed_role(app, parent_org, "Rolle-Dach").await,
        in_sibling_org: seed_role(app, sibling_org, "Rolle-Schwester").await,
    }
}

fn returned_role_ids(body: &serde_json::Value) -> std::collections::BTreeSet<Uuid> {
    body.as_array()
        .unwrap()
        .iter()
        .map(|r| Uuid::parse_str(r["id"].as_str().unwrap()).unwrap())
        .collect()
}

#[tokio::test]
async fn role_list_shows_the_visible_subtree_only_and_excludes_templates() {
    let (harness, app) = spawn_with_auth().await;
    let fixture = seed_role_list_tree(&harness, &app).await;
    // A seeded global template; must never appear regardless of scope.
    let admin_template_id = Uuid::parse_str("01980000-0000-7000-8000-0000000000a1").unwrap();

    let resp = reqwest::Client::new()
        .get(format!("{}/api/v1/roles", app.address))
        .bearer_auth(&fixture.caller_token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let ids = returned_role_ids(&body);
    assert_eq!(ids, fixture.visible_ids());
    assert!(!ids.contains(&fixture.in_parent_org));
    assert!(!ids.contains(&fixture.in_sibling_org));
    assert!(!ids.contains(&admin_template_id));
}

#[tokio::test]
async fn role_list_is_forbidden_without_role_read() {
    let (harness, app) = spawn_with_auth().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let org = seed_child_org(&app, root, "Ohne-Role-Read").await;
    let caller = seed_member(&app, org).await;
    grant_in(&app, caller, org, &["tree:read"]).await;
    let token = harness.sign_token(json!({ "sub": caller.to_string() }));

    let resp = reqwest::Client::new()
        .get(format!("{}/api/v1/roles", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
}
