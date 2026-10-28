use sqlx::PgPool;
use uuid::Uuid;

use crate::helpers::{spawn_app, spawn_app_on_migrated_until};

const ROOT_ORG_ID: &str = "01980000-0000-7000-8000-000000000001";
const BEFORE_WASTE_ROLE_TEMPLATES: i64 = 20261028100200;

async fn permissions_of(pool: &PgPool, role_id: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT permissions FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn waste_role_template_reaches_existing_organizations_and_unedited_roles() {
    let org = Uuid::now_v7();
    let admin_copy = Uuid::now_v7();
    let edited_observer = Uuid::now_v7();
    let untouched_observer = Uuid::now_v7();

    let app = spawn_app_on_migrated_until(BEFORE_WASTE_ROLE_TEMPLATES, async |pool: &PgPool| {
        sqlx::query("INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, 'Waste Org')")
            .bind(org)
            .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
            .execute(pool)
            .await
            .unwrap();
        for (id, name, key, permissions) in [
            (
                admin_copy,
                "Administrator",
                Some("administrator"),
                vec!["sensor:read"],
            ),
            (
                untouched_observer,
                "Beobachter",
                Some("observer"),
                vec!["sensor:read"],
            ),
            (
                edited_observer,
                "Beobachter (eigen)",
                None,
                vec!["sensor:read"],
            ),
        ] {
            sqlx::query(
                "INSERT INTO roles (id, organization_id, name, permissions, template_key)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(org)
            .bind(name)
            .bind(
                permissions
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>(),
            )
            .bind(key)
            .execute(pool)
            .await
            .unwrap();
        }
    })
    .await;

    let admin = permissions_of(&app.db_pool, admin_copy).await;
    assert!(admin.contains(&"waste_bin:create".to_string()));
    assert!(admin.contains(&"collection_plan:delete".to_string()));
    assert!(admin.contains(&"sensor:read".to_string()));

    let observer = permissions_of(&app.db_pool, untouched_observer).await;
    assert!(observer.contains(&"waste_bin:read".to_string()));
    assert!(observer.contains(&"collection_plan:read".to_string()));
    assert!(!observer.contains(&"waste_bin:create".to_string()));

    assert_eq!(
        permissions_of(&app.db_pool, edited_observer).await,
        ["sensor:read"]
    );

    let organizations_without_copy: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM organizations o WHERE
         (SELECT count(*) FROM roles r
           WHERE r.organization_id = o.id AND r.template_key = 'waste_collection') <> 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(organizations_without_copy, 0);

    let templates: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM roles WHERE organization_id IS NULL AND template_key = 'waste_collection'",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(templates, 1);
}

#[tokio::test]
async fn a_new_organization_gets_a_waste_collection_copy() {
    let app = spawn_app().await;
    let org: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": "Neue Orga", "parent_id": ROOT_ORG_ID }),
        )
        .await
        .json()
        .await
        .unwrap();
    let org_id = org["id"].as_str().unwrap();

    let roles: serde_json::Value = app
        .get(&format!("/api/v1/organizations/{org_id}/roles"))
        .await
        .json()
        .await
        .unwrap();
    let copy = roles
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "Abfallsammlung")
        .expect("organization has no Abfallsammlung copy");
    let permissions = copy["permissions"].as_array().unwrap();
    assert!(permissions.iter().any(|p| p == "waste_bin:create"));
    assert!(permissions.iter().any(|p| p == "collection_plan:read"));
}
