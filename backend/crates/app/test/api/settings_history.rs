use domain::{
    Id,
    organization::Organization,
    settings::{Patch, SensorOfflineAfter, SettingKey, SettingsUpdate},
};
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID, settings_repo::child_org};

async fn history_rows(
    app: &crate::helpers::TestApp,
    org: Uuid,
) -> Vec<(String, Option<serde_json::Value>, Option<serde_json::Value>)> {
    sqlx::query!(
        r#"SELECT setting_key AS "key!", previous_value, new_value
           FROM organization_settings_history
           WHERE organization_id = $1
           ORDER BY changed_at ASC, setting_key ASC, id ASC"#,
        org
    )
    .fetch_all(&app.db_pool)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.key, r.previous_value, r.new_value))
    .collect()
}

async fn set_offline_after(app: &crate::helpers::TestApp, org: Uuid, secs: i64) {
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
        .unwrap();
}

#[tokio::test]
async fn each_change_writes_exactly_one_entry() {
    let app = spawn_app().await;
    let org = child_org(&app, "Historie", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_offline_after(&app, org, 90_000).await;
    set_offline_after(&app, org, 120_000).await;

    let rows = history_rows(&app, org).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, SettingKey::SensorOfflineAfter.as_str());
    assert_eq!(rows[0].1, None);
    assert_eq!(rows[1].1, Some(serde_json::json!(90_000)));
    assert_eq!(rows[1].2, Some(serde_json::json!(120_000)));
}

#[tokio::test]
async fn saving_the_same_value_again_writes_nothing() {
    let app = spawn_app().await;
    let org = child_org(&app, "Unveraendert", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_offline_after(&app, org, 90_000).await;
    set_offline_after(&app, org, 90_000).await;

    assert_eq!(history_rows(&app, org).await.len(), 1);
}

#[tokio::test]
async fn returning_to_inheritance_is_visible_in_the_history() {
    let app = spawn_app().await;
    let org = child_org(&app, "Zurueck", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_offline_after(&app, org, 90_000).await;
    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(org),
            SettingsUpdate {
                sensor_offline_after: Patch::Clear,
                ..SettingsUpdate::default()
            },
        )
        .await
        .unwrap();

    let rows = history_rows(&app, org).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].1, Some(serde_json::json!(90_000)));
    assert_eq!(rows[1].2, None);
}

/// Fails inside the write, on the history insert, and must take the value with
/// it. Without a shared transaction the new value would survive without a record.
#[tokio::test]
async fn a_write_that_fails_on_the_history_keeps_the_old_value() {
    let app = spawn_app().await;
    let org = child_org(&app, "Halb", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;
    let unknown_actor = Uuid::now_v7();

    let result = app
        .state
        .settings_service
        .update(
            unknown_actor,
            Id::<Organization>::new(org),
            SettingsUpdate {
                sensor_offline_after: Patch::Set(SensorOfflineAfter::new(90_000).unwrap()),
                ..SettingsUpdate::default()
            },
        )
        .await;

    assert!(result.is_err());
    assert!(history_rows(&app, org).await.is_empty());

    // The rollback must leave no row at all, not a row with a NULL value.
    let stored = sqlx::query_scalar!(
        r#"SELECT sensor_offline_after_secs FROM organization_settings WHERE organization_id = $1"#,
        org
    )
    .fetch_optional(&app.db_pool)
    .await
    .unwrap();
    assert!(stored.is_none());
}

#[tokio::test]
async fn falling_back_by_sending_the_inherited_value_is_recorded() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let org = child_org(&app, "Zurück", root).await;

    set_offline_after(&app, org, 90_000).await;
    // 86_400 is the instance default, which is what `org` inherits.
    set_offline_after(&app, org, 86_400).await;

    let rows = history_rows(&app, org).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].1, Some(serde_json::json!(90_000)));
    assert_eq!(rows[1].2, None, "the own value is gone, not replaced");
}
