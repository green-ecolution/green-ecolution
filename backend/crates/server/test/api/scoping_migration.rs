use serde_json::json;

use crate::helpers::spawn_app;
use crate::organizations::ROOT_ORG_ID;

#[tokio::test]
async fn inserting_a_tree_without_organization_id_fails_without_a_default() {
    let app = spawn_app().await;

    let result = sqlx::query!(
        r#"
        INSERT INTO trees (id, number, species, planting_year, latitude, longitude)
        VALUES ($1, 'NO-ORG-001', 'Quercus robur', 2024, 54.79, 9.44)
        "#,
        uuid::Uuid::now_v7(),
    )
    .execute(&app.db_pool)
    .await;

    let err = result.expect_err("insert must fail: organization_id has no default anymore");
    assert!(
        err.to_string().contains("organization_id"),
        "expected a not-null violation on organization_id, got: {err}"
    );
}

#[tokio::test]
async fn deleting_an_organization_with_a_tree_returns_409() {
    let app = spawn_app().await;
    let org: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &json!({ "name": "Hat einen Baum", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = org["id"].as_str().unwrap();

    let tree = json!({
        "species": "Quercus robur", "number": "SCOPE-MIG-001", "planting_year": 2024,
        "latitude": 54.79, "longitude": 9.44, "description": "",
        "organization_id": org_id
    });
    let resp = app.post_json("/api/v1/trees", &tree).await;
    assert_eq!(resp.status(), 201);

    let resp = app.delete(&format!("/api/v1/organizations/{org_id}")).await;
    assert_eq!(resp.status(), 409);
}
