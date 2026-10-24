use std::collections::BTreeMap;
use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    module::{ModuleActivationReader, ModuleActivationWriter, ModuleKey},
    organization::Organization,
    settings::SettingOrigin,
};
use server::infra::{
    pg_module_activation::PgModuleActivationRepository, pg_organization::PgOrganizationRepository,
};
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID, settings_repo::child_org};

fn key(k: &str) -> ModuleKey {
    ModuleKey::new(k).unwrap()
}

fn repo(app: &crate::helpers::TestApp, defaults: &[(&str, bool)]) -> PgModuleActivationRepository {
    PgModuleActivationRepository::new(
        app.db_pool.clone(),
        Arc::new(PgOrganizationRepository::new(app.db_pool.clone())),
        defaults.iter().map(|(k, v)| (key(k), *v)).collect(),
    )
}

async fn lock(app: &crate::helpers::TestApp, org: Uuid) {
    sqlx::query!(
        r#"INSERT INTO organization_settings (organization_id, descendants_may_override)
           VALUES ($1, false)
           ON CONFLICT (organization_id) DO UPDATE SET descendants_may_override = false"#,
        org
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn activation_inherits_over_three_levels_and_own_values_win() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Mitte", root).await;
    let leaf = child_org(&app, "Blatt", mid).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    repo.apply(Id::new(root), &BTreeMap::from([(key("alpha"), Some(true))]))
        .await
        .unwrap();
    repo.apply(Id::new(leaf), &BTreeMap::from([(key("beta"), Some(false))]))
        .await
        .unwrap();

    let at_leaf = repo.resolve(Id::<Organization>::new(leaf)).await.unwrap();
    assert!(at_leaf[&key("alpha")].enabled);
    assert_eq!(
        at_leaf[&key("alpha")].origin,
        SettingOrigin::Inherited(Id::new(root))
    );
    assert!(!at_leaf[&key("beta")].enabled);
    assert_eq!(at_leaf[&key("beta")].origin, SettingOrigin::Own);

    let at_mid = repo.resolve(Id::new(mid)).await.unwrap();
    assert!(at_mid[&key("beta")].enabled);
    assert_eq!(at_mid[&key("beta")].origin, SettingOrigin::Default);

    assert_eq!(
        repo.own(Id::new(leaf)).await.unwrap(),
        BTreeMap::from([(key("beta"), false)])
    );
}

#[tokio::test]
async fn clearing_returns_to_inheritance() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let org = child_org(&app, "Klar", root).await;
    let repo = repo(&app, &[("alpha", false)]);

    repo.apply(Id::new(org), &BTreeMap::from([(key("alpha"), Some(true))]))
        .await
        .unwrap();
    repo.apply(Id::new(org), &BTreeMap::from([(key("alpha"), None)]))
        .await
        .unwrap();

    let r = repo.resolve(Id::new(org)).await.unwrap();
    assert!(!r[&key("alpha")].enabled);
    assert_eq!(r[&key("alpha")].origin, SettingOrigin::Default);
    assert!(repo.own(Id::new(org)).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_lock_above_freezes_the_subtree() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Sperre", root).await;
    let leaf = child_org(&app, "Darunter", mid).await;
    let repo = repo(&app, &[("alpha", false)]);

    repo.apply(Id::new(mid), &BTreeMap::from([(key("alpha"), Some(true))]))
        .await
        .unwrap();
    repo.apply(
        Id::new(leaf),
        &BTreeMap::from([(key("alpha"), Some(false))]),
    )
    .await
    .unwrap();
    lock(&app, mid).await;

    let r = repo.resolve(Id::new(leaf)).await.unwrap();
    assert!(r[&key("alpha")].enabled);
    assert_eq!(r[&key("alpha")].enforced_by, Some(Id::new(mid)));
}

#[tokio::test]
async fn active_orgs_lists_exactly_the_enabled_organizations() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let on = child_org(&app, "An", root).await;
    let off = child_org(&app, "Aus", root).await;
    let repo = repo(&app, &[("alpha", false)]);

    repo.apply(Id::new(on), &BTreeMap::from([(key("alpha"), Some(true))]))
        .await
        .unwrap();

    let active = repo.active_orgs(&key("alpha")).await.unwrap();
    assert!(active.contains(&Id::new(on)));
    assert!(!active.contains(&Id::new(off)));
    assert!(!active.contains(&Id::new(root)));
    assert!(
        repo.active_orgs(&key("unregistered"))
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn unknown_organization_is_not_found() {
    let app = spawn_app().await;
    let repo = repo(&app, &[("alpha", false)]);
    assert!(matches!(
        repo.resolve(Id::new_v7()).await,
        Err(RepositoryError::NotFound)
    ));
}
