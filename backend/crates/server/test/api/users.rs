use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};
use serde_json::json;
use uuid::Uuid;

async fn create_org(app: &crate::helpers::TestApp, name: &str) -> String {
    let resp = app
        .post_json(
            "/api/v1/organizations",
            &json!({ "name": name, "parent_id": ROOT_ORG_ID }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_role(app: &crate::helpers::TestApp, org: &str, name: &str) -> String {
    let resp = app
        .post_json(
            &format!("/api/v1/organizations/{org}/roles"),
            &json!({ "name": name, "permissions": ["tree:read"] }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn valid_body() -> serde_json::Value {
    serde_json::json!({
        "employee_id": "EMP-1",
        "phone_number": "+49 461 1",
        "avatar_url": "",
        "status": "absent",
        "driving_licenses": ["B", "CE"]
    })
}

#[tokio::test]
async fn get_me_returns_the_authenticated_user() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/users/me").await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["username"], "ttester");
    assert_eq!(body["id"], "00000000-0000-0000-0000-000000000000");
}

#[tokio::test]
async fn update_user_returns_200_for_demo_user() {
    let app = spawn_app().await;

    let response = app
        .put_json(
            "/api/v1/users/00000000-0000-0000-0000-000000000000",
            &valid_body(),
        )
        .await;

    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["username"], "ttester");
}

#[tokio::test]
async fn update_user_returns_404_for_unknown_user() {
    let app = spawn_app().await;

    let response = app
        .put_json(
            &format!("/api/v1/users/{}", uuid::Uuid::now_v7()),
            &valid_body(),
        )
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn update_user_returns_400_for_invalid_avatar_url() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "avatar_url": "not a url",
        "status": "absent",
        "driving_licenses": []
    });
    let response = app
        .put_json("/api/v1/users/00000000-0000-0000-0000-000000000000", &body)
        .await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn assign_and_revoke_role_via_api() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let role_id = create_role(&app, &org, "Gießtrupp").await;
    let user_id = Uuid::new_v4();

    // Creating the organization membership implicitly creates the user_profiles row.
    let resp = app
        .patch_json(
            &format!("/api/v1/users/{user_id}/organization"),
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let resp = app
        .post_json(
            &format!("/api/v1/users/{user_id}/roles"),
            &json!({ "role_id": role_id }),
        )
        .await;
    assert_eq!(resp.status(), 201);

    let roles: serde_json::Value = app
        .get(&format!("/api/v1/users/{user_id}/roles"))
        .await
        .json()
        .await
        .unwrap();
    assert!(
        roles
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == role_id.as_str())
    );

    let resp = app
        .delete(&format!("/api/v1/users/{user_id}/roles/{role_id}"))
        .await;
    assert_eq!(resp.status(), 204);

    let roles: serde_json::Value = app
        .get(&format!("/api/v1/users/{user_id}/roles"))
        .await
        .json()
        .await
        .unwrap();
    assert!(roles.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_users_accepts_pagination_query_params() {
    let app = spawn_app().await;
    // Regression: serde(flatten) over PaginationParams rejected numeric query
    // values ("invalid type: string \"1\", expected u64").
    let resp = app.get("/api/v1/users?page=1&per_page=100").await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn assigning_a_template_role_returns_409() {
    let app = spawn_app().await;
    let user_id = Uuid::new_v4();
    let resp = app
        .post_json(
            &format!("/api/v1/users/{user_id}/roles"),
            &json!({ "role_id": "01980000-0000-7000-8000-0000000000a1" }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn create_user_with_template_role_returns_409() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let org_uuid = Uuid::parse_str(&org).unwrap();

    let resp = app
        .post_json(
            "/api/v1/users",
            &json!({
                "username": "newuser",
                "first_name": "New",
                "last_name": "User",
                "email": "newuser@example.com",
                "password": "SecurePass123!",
                "organization_id": org_uuid,
                "role_ids": ["01980000-0000-7000-8000-0000000000a1"]
            }),
        )
        .await;
    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn set_organization_persists() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let user_id = Uuid::new_v4();

    let resp = app
        .patch_json(
            &format!("/api/v1/users/{user_id}/organization"),
            &json!({ "organization_id": org }),
        )
        .await;
    assert_eq!(resp.status(), 200);

    let stored = sqlx::query_scalar!(
        r#"SELECT organization_id FROM user_profiles WHERE id = $1"#,
        user_id
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored, Some(Uuid::parse_str(&org).unwrap()));
}

#[tokio::test]
async fn list_users_filtered_by_role_uses_db_assignments() {
    let app = spawn_app().await;
    let org = create_org(&app, "TBZ").await;
    let org_uuid = Uuid::parse_str(&org).unwrap();
    let role_id = Uuid::parse_str(&create_role(&app, &org, "Gießtrupp").await).unwrap();
    let user_id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)"#,
        user_id,
        org_uuid
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

    let resp = app.get(&format!("/api/v1/users?role_id={role_id}")).await;
    assert_eq!(resp.status(), 200);
    // In demo mode the Keycloak path resolves no identities, so the DB-derived
    // id set yields an empty page (deterministic).
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["data"].as_array().unwrap().is_empty());
}
