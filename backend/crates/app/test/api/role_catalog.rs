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
