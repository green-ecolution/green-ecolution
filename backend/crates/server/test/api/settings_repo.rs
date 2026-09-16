use domain::{Id, organization::Organization, settings::SettingOrigin};
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

pub(crate) async fn child_org(app: &crate::helpers::TestApp, name: &str, parent: Uuid) -> Uuid {
    let created: serde_json::Value = app
        .post_json(
            "/api/v1/organizations",
            &serde_json::json!({ "name": name, "parent_id": parent.to_string() }),
        )
        .await
        .json()
        .await
        .unwrap();
    Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
}

pub(crate) async fn set_water_demand(app: &crate::helpers::TestApp, org: Uuid, liters: f64) {
    sqlx::query!(
        r#"INSERT INTO organization_settings (organization_id, water_demand_liters)
           VALUES ($1, $2)
           ON CONFLICT (organization_id)
           DO UPDATE SET water_demand_liters = EXCLUDED.water_demand_liters"#,
        org,
        liters,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn a_leaf_without_a_value_inherits_from_the_nearest_ancestor_that_has_one() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Mitte", root).await;
    let leaf = child_org(&app, "Blatt", mid).await;

    set_water_demand(&app, root, 100.0).await;
    set_water_demand(&app, mid, 90.0).await;

    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();

    assert_eq!(resolution.effective.water_demand.liters(), 90.0);
    assert_eq!(
        resolution.origins.water_demand,
        SettingOrigin::Inherited(Id::new(mid))
    );
}

/// An organization created after an earlier resolution must still resolve
/// against its parent: the adapter is told nothing when an organization
/// appears, so it may remember nothing between reads.
#[tokio::test]
async fn an_organization_created_after_the_first_read_still_inherits() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    set_water_demand(&app, root, 100.0).await;

    let warm = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(root))
        .await
        .unwrap();
    assert_eq!(warm.effective.water_demand.liters(), 100.0);

    let fresh = child_org(&app, "Spaet gegruendet", root).await;
    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(fresh))
        .await
        .unwrap();

    assert_eq!(resolution.effective.water_demand.liters(), 100.0);
    assert_eq!(
        resolution.origins.water_demand,
        SettingOrigin::Inherited(Id::new(root))
    );
}

#[tokio::test]
async fn an_organization_without_any_value_gets_the_instance_default() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Ohne", root).await;

    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();

    assert_eq!(resolution.effective.water_demand.liters(), 80.0);
    assert_eq!(resolution.origins.water_demand, SettingOrigin::Default);
}

#[tokio::test]
async fn an_own_value_beats_the_inherited_one() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Eigen", root).await;

    set_water_demand(&app, root, 100.0).await;
    set_water_demand(&app, leaf, 70.0).await;

    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();

    assert_eq!(resolution.effective.water_demand.liters(), 70.0);
    assert_eq!(resolution.origins.water_demand, SettingOrigin::Own);
}

pub(crate) async fn set_just_watered_ttl(app: &crate::helpers::TestApp, org: Uuid, secs: i64) {
    sqlx::query!(
        r#"INSERT INTO organization_settings (organization_id, just_watered_ttl_secs)
           VALUES ($1, $2)
           ON CONFLICT (organization_id)
           DO UPDATE SET just_watered_ttl_secs = EXCLUDED.just_watered_ttl_secs"#,
        org,
        secs,
    )
    .execute(&app.db_pool)
    .await
    .unwrap();
}
