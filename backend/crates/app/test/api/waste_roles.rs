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

const WASTE_ADMIN_PERMISSIONS: [&str; 8] = [
    "collection_plan:create",
    "collection_plan:delete",
    "collection_plan:read",
    "collection_plan:update",
    "waste_bin:create",
    "waste_bin:delete",
    "waste_bin:read",
    "waste_bin:update",
];

async fn template_permissions(pool: &PgPool, key: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT permissions FROM roles WHERE organization_id IS NULL AND template_key = $1",
    )
    .bind(key)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn insert_role(
    pool: &PgPool,
    id: Uuid,
    org: Uuid,
    name: &str,
    key: Option<&str>,
    permissions: Vec<String>,
) {
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions, template_key)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(org)
    .bind(name)
    .bind(permissions)
    .bind(key)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_organization(pool: &PgPool, id: Uuid, name: &str) {
    sqlx::query("INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
}

fn sorted(mut permissions: Vec<String>) -> Vec<String> {
    permissions.sort();
    permissions
}

#[tokio::test]
async fn waste_role_template_reaches_existing_organizations_and_unedited_roles() {
    let org = Uuid::now_v7();
    let admin_copy = Uuid::now_v7();
    let trimmed_admin = Uuid::now_v7();
    let hand_made_observer = Uuid::now_v7();
    let untouched_observer = Uuid::now_v7();
    let trimmed_admin_org = Uuid::now_v7();

    let app = spawn_app_on_migrated_until(BEFORE_WASTE_ROLE_TEMPLATES, async |pool: &PgPool| {
        insert_organization(pool, org, "Waste Org").await;
        insert_organization(pool, trimmed_admin_org, "Trimmed Org").await;
        let admin_template = template_permissions(pool, "administrator").await;
        let observer_template = template_permissions(pool, "observer").await;
        insert_role(
            pool,
            admin_copy,
            org,
            "Administrator",
            Some("administrator"),
            admin_template.iter().rev().cloned().collect(),
        )
        .await;
        insert_role(
            pool,
            untouched_observer,
            org,
            "Beobachter",
            Some("observer"),
            observer_template,
        )
        .await;
        insert_role(
            pool,
            hand_made_observer,
            org,
            "Beobachter (eigen)",
            None,
            vec!["sensor:read".to_string()],
        )
        .await;
        // Permissions trimmed through the API keep the template key.
        insert_role(
            pool,
            trimmed_admin,
            trimmed_admin_org,
            "Administrator",
            Some("administrator"),
            vec!["sensor:read".to_string(), "vehicle:read".to_string()],
        )
        .await;
    })
    .await;

    let admin = permissions_of(&app.db_pool, admin_copy).await;
    for permission in WASTE_ADMIN_PERMISSIONS {
        assert!(admin.contains(&permission.to_string()), "{permission}");
    }
    assert!(admin.contains(&"sensor:read".to_string()));

    let observer = permissions_of(&app.db_pool, untouched_observer).await;
    assert!(observer.contains(&"waste_bin:read".to_string()));
    assert!(observer.contains(&"collection_plan:read".to_string()));
    assert!(!observer.contains(&"waste_bin:create".to_string()));

    assert_eq!(
        permissions_of(&app.db_pool, hand_made_observer).await,
        ["sensor:read"]
    );
    assert_eq!(
        permissions_of(&app.db_pool, trimmed_admin).await,
        ["sensor:read", "vehicle:read"]
    );

    let admin_template = template_permissions(&app.db_pool, "administrator").await;
    for permission in WASTE_ADMIN_PERMISSIONS {
        assert!(
            admin_template.contains(&permission.to_string()),
            "{permission}"
        );
    }
    let observer_template = template_permissions(&app.db_pool, "observer").await;
    assert!(observer_template.contains(&"waste_bin:read".to_string()));
    assert!(observer_template.contains(&"collection_plan:read".to_string()));
    assert!(!observer_template.contains(&"waste_bin:update".to_string()));

    assert_eq!(
        sorted(template_permissions(&app.db_pool, "waste_collection").await),
        [
            "collection_plan:create",
            "collection_plan:delete",
            "collection_plan:read",
            "collection_plan:update",
            "region:read",
            "sensor:read",
            "start_point:read",
            "vehicle:read",
            "waste_bin:create",
            "waste_bin:delete",
            "waste_bin:read",
            "waste_bin:update",
        ]
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
async fn an_organization_with_its_own_abfallsammlung_role_keeps_it() {
    let org = Uuid::now_v7();
    let own_role = Uuid::now_v7();

    let app = spawn_app_on_migrated_until(BEFORE_WASTE_ROLE_TEMPLATES, async |pool: &PgPool| {
        insert_organization(pool, org, "Eigene Rolle Org").await;
        insert_role(
            pool,
            own_role,
            org,
            "Abfallsammlung",
            None,
            vec!["vehicle:read".to_string()],
        )
        .await;
    })
    .await;

    let roles: Vec<(Uuid, Option<String>)> = sqlx::query_as(
        "SELECT id, template_key FROM roles WHERE organization_id = $1 AND name = 'Abfallsammlung'",
    )
    .bind(org)
    .fetch_all(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(roles, [(own_role, None)]);
    assert_eq!(
        permissions_of(&app.db_pool, own_role).await,
        ["vehicle:read"]
    );
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
