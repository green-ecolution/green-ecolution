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

pub(crate) async fn set_sensor_offline_after(app: &crate::helpers::TestApp, org: Uuid, secs: i64) {
    sqlx::query!(
        r#"INSERT INTO organization_settings (organization_id, sensor_offline_after_secs)
           VALUES ($1, $2)
           ON CONFLICT (organization_id)
           DO UPDATE SET sensor_offline_after_secs = EXCLUDED.sensor_offline_after_secs"#,
        org,
        secs,
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

    set_sensor_offline_after(&app, root, 100_000).await;
    set_sensor_offline_after(&app, mid, 90_000).await;

    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();

    assert_eq!(resolution.effective.sensor_offline_after.seconds(), 90_000);
    assert_eq!(
        resolution.origins.sensor_offline_after,
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
    set_sensor_offline_after(&app, root, 100_000).await;

    let warm = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(root))
        .await
        .unwrap();
    assert_eq!(warm.effective.sensor_offline_after.seconds(), 100_000);

    let fresh = child_org(&app, "Spaet gegruendet", root).await;
    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(fresh))
        .await
        .unwrap();

    assert_eq!(resolution.effective.sensor_offline_after.seconds(), 100_000);
    assert_eq!(
        resolution.origins.sensor_offline_after,
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

    assert_eq!(resolution.effective.sensor_offline_after.seconds(), 86_400);
    assert_eq!(
        resolution.origins.sensor_offline_after,
        SettingOrigin::Default
    );
}

#[tokio::test]
async fn an_own_value_beats_the_inherited_one() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Eigen", root).await;

    set_sensor_offline_after(&app, root, 100_000).await;
    set_sensor_offline_after(&app, leaf, 70_000).await;

    let resolution = app
        .state
        .settings_reader
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();

    assert_eq!(resolution.effective.sensor_offline_after.seconds(), 70_000);
    assert_eq!(resolution.origins.sensor_offline_after, SettingOrigin::Own);
}

pub(crate) async fn set_just_watered_ttl(app: &crate::helpers::TestApp, org: Uuid, secs: i64) {
    sqlx::query!(
        r#"INSERT INTO irrigation_settings (organization_id, just_watered_ttl_secs)
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
