use domain::plugin::{PluginDraft, PluginName, PluginReader, PluginSlug, PluginWriter};
use server::infra::pg_plugin::PgPluginRepository;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::helpers::spawn_app;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

fn draft(slug: &str) -> PluginDraft {
    PluginDraft {
        slug: PluginSlug::new(slug).unwrap(),
        name: PluginName::new("Testplugin").unwrap(),
        description: None,
        organization_id: domain::Id::new(Uuid::parse_str(ROOT_ORG).unwrap()),
        permissions: BTreeSet::new(),
        required_permissions: BTreeSet::new(),
        frontend: domain::plugin::PluginFrontend::None,
    }
}

#[tokio::test]
async fn save_new_then_load_by_slug() {
    let app = spawn_app().await;
    let repo = PgPluginRepository::new(app.db_pool.clone());

    let id = domain::Id::new_v7();
    let saved = repo.save_new(id, draft("acme"), None).await.unwrap();
    let loaded = repo
        .by_slug(&PluginSlug::new("acme").unwrap())
        .await
        .unwrap();

    assert_eq!(loaded.id, saved.id);
    assert!(!loaded.enabled(), "a new plugin starts disabled");
    assert!(loaded.key_hash().is_none());
}

#[tokio::test]
async fn tree_refs_pages_over_the_keyset() {
    let app = spawn_app().await;
    let repo = PgPluginRepository::new(app.db_pool.clone());
    let plugin = repo
        .save_new(domain::Id::new_v7(), draft("pager"), None)
        .await
        .unwrap();

    for i in 0..5 {
        let tree_id = insert_tree(&app.db_pool).await;
        repo.link_tree(plugin.id, &format!("ext-{i}"), domain::Id::new(tree_id))
            .await
            .unwrap();
    }

    let first = repo.tree_refs(plugin.id, None, 2).await.unwrap();
    assert_eq!(first.items.len(), 2);
    assert_eq!(first.items[0].external_id, "ext-0");

    let second = repo
        .tree_refs(plugin.id, Some(&first.items[1].external_id), 2)
        .await
        .unwrap();
    assert_eq!(second.items[0].external_id, "ext-2");
}

#[tokio::test]
async fn deleting_a_tree_removes_its_ref() {
    let app = spawn_app().await;
    let repo = PgPluginRepository::new(app.db_pool.clone());
    let plugin = repo
        .save_new(domain::Id::new_v7(), draft("cascade"), None)
        .await
        .unwrap();
    let tree_id = insert_tree(&app.db_pool).await;
    repo.link_tree(plugin.id, "ext-1", domain::Id::new(tree_id))
        .await
        .unwrap();

    sqlx::query!("DELETE FROM trees WHERE id = $1", tree_id)
        .execute(&app.db_pool)
        .await
        .unwrap();

    assert!(
        repo.tree_ref(plugin.id, "ext-1").await.unwrap().is_none(),
        "the FK must cascade so no ref outlives its tree"
    );
}

async fn insert_tree(pool: &sqlx::PgPool) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"INSERT INTO trees (id, number, species, planting_year, latitude, longitude, geometry,
                              watering_status, organization_id)
           VALUES ($1, 'T-1', 'Quercus', 2000, 54.78, 9.43,
                   ST_SetSRID(ST_MakePoint(9.43, 54.78), 4326), 'unknown', $2)"#,
        id,
        Uuid::parse_str(ROOT_ORG).unwrap()
    )
    .execute(pool)
    .await
    .expect("failed to insert tree");
    id
}
