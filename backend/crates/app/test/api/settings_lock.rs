use domain::{
    Id,
    organization::Organization,
    settings::{Patch, SensorOfflineAfter, SettingsUpdate},
};
use server::service::ServiceError;
use uuid::Uuid;

use crate::{
    helpers::spawn_app,
    organizations::ROOT_ORG_ID,
    settings_repo::{child_org, set_sensor_offline_after},
};

async fn lock(app: &crate::helpers::TestApp, org: Uuid) {
    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(org),
            SettingsUpdate {
                descendants_may_override: Some(false),
                ..SettingsUpdate::default()
            },
        )
        .await
        .expect("locking must be allowed at the locking level");
}

async fn set_via_service(
    app: &crate::helpers::TestApp,
    org: Uuid,
    secs: i64,
) -> Result<(), ServiceError> {
    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(org),
            SettingsUpdate {
                sensor_offline_after: Patch::Set(SensorOfflineAfter::new(secs).unwrap()),
                ..SettingsUpdate::default()
            },
        )
        .await
        .map(|_| ())
}

#[tokio::test]
async fn a_write_below_a_lock_is_rejected_as_a_conflict() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    set_sensor_offline_after(&app, root, 100_000).await;
    lock(&app, root).await;

    let err = set_via_service(&app, leaf, 70_000).await.unwrap_err();
    assert_eq!(err.code(), "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn the_lock_reaches_through_two_levels() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let mid = child_org(&app, "Mitte", root).await;
    let leaf = child_org(&app, "Blatt", mid).await;

    lock(&app, root).await;

    assert_eq!(
        set_via_service(&app, leaf, 70_000)
            .await
            .unwrap_err()
            .code(),
        "settings.enforced_by_ancestor"
    );
}

#[tokio::test]
async fn a_sub_unit_cannot_unlock_itself() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    lock(&app, root).await;

    let err = app
        .state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(leaf),
            SettingsUpdate {
                descendants_may_override: Some(true),
                ..SettingsUpdate::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err.code(), "settings.enforced_by_ancestor");
}

#[tokio::test]
async fn a_dormant_value_comes_back_unchanged_when_the_lock_is_released() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Blatt", root).await;

    set_sensor_offline_after(&app, root, 100_000).await;
    set_via_service(&app, leaf, 70_000).await.unwrap();
    lock(&app, root).await;

    let during = app
        .state
        .settings_service
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();
    assert_eq!(during.effective.sensor_offline_after.seconds(), 100_000);
    assert_eq!(during.effective.enforced_by, Some(Id::new(root)));

    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(root),
            SettingsUpdate {
                descendants_may_override: Some(true),
                ..SettingsUpdate::default()
            },
        )
        .await
        .unwrap();

    let after = app
        .state
        .settings_service
        .resolution(Id::<Organization>::new(leaf))
        .await
        .unwrap();
    assert_eq!(after.effective.sensor_offline_after.seconds(), 70_000);
    assert_eq!(after.effective.enforced_by, None);
}
