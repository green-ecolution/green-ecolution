use std::sync::Arc;

use domain::authorization::{Action, Permission, ResourceCatalog, ResourceKey};
use domain::role::RoleReader;
use server::infra::pg_role::PgRoleRepository;
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

#[tokio::test]
async fn a_stored_permission_of_an_unknown_resource_is_ignored() {
    let app = spawn_app().await;
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, description, permissions)
         VALUES ($1, $2, 'ghosts', '', $3)",
    )
    .bind(role_id)
    .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
    .bind(vec!["sensor:read".to_string(), "ghost:read".to_string()])
    .execute(&app.db_pool)
    .await
    .unwrap();

    let repo = PgRoleRepository::new(
        app.db_pool.clone(),
        Arc::new(ResourceCatalog::new(server::irrigation::resources::ALL).unwrap()),
    );
    let role = repo.by_id(domain::Id::new(role_id)).await.unwrap();

    assert_eq!(
        role.permissions().iter().copied().collect::<Vec<_>>(),
        vec![Permission::new(ResourceKey::SENSOR, Action::Read)]
    );
}

#[tokio::test]
async fn holders_of_watering_plan_rights_receive_the_matching_start_point_rights() {
    let app = crate::helpers::spawn_app_on_migrated_until(
        crate::helpers::BEFORE_START_POINT_RESOURCE,
        async |pool: &sqlx::PgPool| {
            sqlx::query("INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'planner', $3)")
                .bind(Uuid::now_v7())
                .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
                .bind(vec!["watering_plan:read".to_string(), "watering_plan:update".to_string()])
                .execute(pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO plugins (id, slug, name, organization_id, permissions, required_permissions) VALUES ($1, 'p', 'P', $2, $3, $4)")
                .bind(Uuid::now_v7())
                .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
                .bind(vec!["watering_plan:read".to_string()])
                .bind(vec!["watering_plan:update".to_string()])
                .execute(pool)
                .await
                .unwrap();
        },
    )
    .await;

    let mut role: Vec<String> =
        sqlx::query_scalar("SELECT permissions FROM roles WHERE name = 'planner'")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    role.sort();
    assert_eq!(
        role,
        [
            "start_point:read",
            "start_point:update",
            "watering_plan:read",
            "watering_plan:update"
        ]
    );

    let (granted, required): (Vec<String>, Vec<String>) =
        sqlx::query_as("SELECT permissions, required_permissions FROM plugins WHERE slug = 'p'")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(granted, ["watering_plan:read", "start_point:read"]);
    assert_eq!(required, ["watering_plan:update"]);

    let templates_with_plan: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM roles WHERE organization_id IS NULL
         AND permissions::text LIKE '%watering_plan:read%'",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert!(
        templates_with_plan > 0,
        "no template holds watering_plan:read"
    );

    let templates_missing_start_point: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM roles WHERE organization_id IS NULL
         AND permissions::text LIKE '%watering_plan:read%'
         AND NOT permissions @> ARRAY['start_point:read']",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(templates_missing_start_point, 0);
}
