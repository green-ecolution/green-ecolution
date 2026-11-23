use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    module::{ModuleActivationReader, ModuleActivationWriter, ModuleKey, ModuleState},
    organization::Organization,
};
use server::infra::{
    pg_module_activation::PgModuleActivationRepository, pg_organization::PgOrganizationRepository,
};
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID, settings_repo::child_org};

const LOCKED: ModuleState = ModuleState {
    released: false,
    enabled: false,
};
const ON: ModuleState = ModuleState {
    released: true,
    enabled: true,
};
const OFF: ModuleState = ModuleState {
    released: true,
    enabled: false,
};

fn key(k: &str) -> ModuleKey {
    ModuleKey::new(k).unwrap()
}

fn org(id: Uuid) -> Id<Organization> {
    Id::new(id)
}

fn repo(app: &crate::helpers::TestApp, defaults: &[(&str, bool)]) -> PgModuleActivationRepository {
    PgModuleActivationRepository::new(
        app.db_pool.clone(),
        Arc::new(PgOrganizationRepository::new(app.db_pool.clone())),
        defaults.iter().map(|(k, v)| (key(k), *v)).collect(),
    )
}

fn changes(entries: &[(&str, bool)]) -> BTreeMap<ModuleKey, bool> {
    entries.iter().map(|(k, v)| (key(k), *v)).collect()
}

#[tokio::test]
async fn a_tenant_without_a_row_has_the_module_locked() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tenant = child_org(&app, "Mandant", root).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    let states = repo.resolve(org(tenant)).await.unwrap();
    assert_eq!(states[&key("beta")], LOCKED);
    assert_eq!(states[&key("alpha")], LOCKED);
    assert_eq!(
        repo.active_orgs(&key("beta")).await.unwrap(),
        BTreeSet::from([org(root)])
    );
    assert_eq!(
        repo.released_orgs(&key("beta")).await.unwrap(),
        BTreeSet::from([org(root)])
    );
}

#[tokio::test]
async fn the_root_follows_the_default_until_it_switches() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tenant = child_org(&app, "Mandant", root).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    let states = repo.resolve(org(root)).await.unwrap();
    assert_eq!(states[&key("alpha")], OFF);
    assert_eq!(states[&key("beta")], ON);

    repo.switch(org(root), &changes(&[("alpha", true)]))
        .await
        .unwrap();
    assert_eq!(repo.resolve(org(root)).await.unwrap()[&key("alpha")], ON);
    assert_eq!(
        repo.resolve(org(tenant)).await.unwrap()[&key("alpha")],
        LOCKED
    );
}

#[tokio::test]
async fn releasing_switches_on_and_reaches_the_sub_organization() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tenant = child_org(&app, "Mandant", root).await;
    let sub = child_org(&app, "Unter", tenant).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    repo.release(org(tenant), &changes(&[("alpha", true)]))
        .await
        .unwrap();

    assert_eq!(repo.resolve(org(tenant)).await.unwrap()[&key("alpha")], ON);
    assert_eq!(repo.resolve(org(sub)).await.unwrap()[&key("alpha")], ON);
    assert_eq!(
        repo.active_orgs(&key("alpha")).await.unwrap(),
        BTreeSet::from([org(tenant), org(sub)])
    );
}

#[tokio::test]
async fn locking_keeps_the_switch_and_releasing_restores_it() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let tenant = child_org(&app, "Mandant", root).await;
    let sub = child_org(&app, "Unter", tenant).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    repo.release(org(tenant), &changes(&[("alpha", true)]))
        .await
        .unwrap();
    repo.switch(org(tenant), &changes(&[("alpha", false)]))
        .await
        .unwrap();
    repo.release(org(tenant), &changes(&[("alpha", false)]))
        .await
        .unwrap();

    let released = repo.released_orgs(&key("alpha")).await.unwrap();
    assert!(!released.contains(&org(tenant)));
    assert!(!released.contains(&org(sub)));
    assert!(!repo.resolve(org(tenant)).await.unwrap()[&key("alpha")].released);

    repo.release(org(tenant), &changes(&[("alpha", true)]))
        .await
        .unwrap();
    assert_eq!(repo.resolve(org(tenant)).await.unwrap()[&key("alpha")], OFF);
}

#[tokio::test]
async fn tenants_lists_every_tenant_with_every_registered_module() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let first = child_org(&app, "Erster", root).await;
    let second = child_org(&app, "Zweiter", root).await;
    let sub = child_org(&app, "Unter", first).await;
    let repo = repo(&app, &[("alpha", false), ("beta", true)]);

    repo.release(org(first), &changes(&[("alpha", true)]))
        .await
        .unwrap();

    let tenants = repo.tenants().await.unwrap();
    assert_eq!(
        tenants.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([org(first), org(second)])
    );
    assert!(!tenants.contains_key(&org(root)));
    assert!(!tenants.contains_key(&org(sub)));
    for states in tenants.values() {
        assert_eq!(
            states.keys().cloned().collect::<Vec<_>>(),
            vec![key("alpha"), key("beta")]
        );
    }
    assert_eq!(tenants[&org(first)][&key("alpha")], ON);
    assert_eq!(tenants[&org(second)][&key("alpha")], LOCKED);
}

#[tokio::test]
async fn unknown_organization_is_not_found() {
    let app = spawn_app().await;
    let repo = repo(&app, &[("alpha", false)]);

    let result = repo.resolve(Id::new(Uuid::now_v7())).await;
    assert!(matches!(result, Err(RepositoryError::NotFound)));
}
