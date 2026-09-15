use domain::{
    Id,
    organization::Organization,
    settings::{Patch, SettingKey, SettingsUpdate, WaterDemand},
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

async fn set_demand(app: &crate::helpers::TestApp, org: Uuid, liters: f64) {
    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(org),
            SettingsUpdate {
                water_demand: Patch::Set(WaterDemand::new(liters).unwrap()),
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

    set_demand(&app, org, 90.0).await;
    set_demand(&app, org, 120.0).await;

    let rows = history_rows(&app, org).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, SettingKey::WaterDemand.as_str());
    assert_eq!(rows[0].1, None);
    assert_eq!(rows[1].1, Some(serde_json::json!(90.0)));
    assert_eq!(rows[1].2, Some(serde_json::json!(120.0)));
}

#[tokio::test]
async fn saving_the_same_value_again_writes_nothing() {
    let app = spawn_app().await;
    let org = child_org(&app, "Unveraendert", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_demand(&app, org, 90.0).await;
    set_demand(&app, org, 90.0).await;

    assert_eq!(history_rows(&app, org).await.len(), 1);
}

#[tokio::test]
async fn returning_to_inheritance_is_visible_in_the_history() {
    let app = spawn_app().await;
    let org = child_org(&app, "Zurueck", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_demand(&app, org, 90.0).await;
    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(org),
            SettingsUpdate {
                water_demand: Patch::Clear,
                ..SettingsUpdate::default()
            },
        )
        .await
        .unwrap();

    let rows = history_rows(&app, org).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].1, Some(serde_json::json!(90.0)));
    assert_eq!(rows[1].2, None);
}

#[tokio::test]
async fn a_rejected_change_leaves_no_entry() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let leaf = child_org(&app, "Gesperrt", root).await;

    app.state
        .settings_service
        .update(
            Uuid::nil(),
            Id::<Organization>::new(root),
            SettingsUpdate {
                descendants_may_override: Some(false),
                ..SettingsUpdate::default()
            },
        )
        .await
        .unwrap();

    assert!(
        app.state
            .settings_service
            .update(
                Uuid::nil(),
                Id::<Organization>::new(leaf),
                SettingsUpdate {
                    water_demand: Patch::Set(WaterDemand::new(70.0).unwrap()),
                    ..SettingsUpdate::default()
                },
            )
            .await
            .is_err()
    );

    assert!(history_rows(&app, leaf).await.is_empty());
}

/// The rejection above never reaches the database; this one fails inside the
/// write, on the history insert, and must take the value with it. Without a
/// shared transaction the new value would survive without a record of it.
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
                water_demand: Patch::Set(WaterDemand::new(90.0).unwrap()),
                ..SettingsUpdate::default()
            },
        )
        .await;

    assert!(result.is_err());
    assert!(history_rows(&app, org).await.is_empty());

    // The rollback must leave no row at all, not a row with a NULL value.
    let stored = sqlx::query_scalar!(
        r#"SELECT water_demand_liters FROM organization_settings WHERE organization_id = $1"#,
        org
    )
    .fetch_optional(&app.db_pool)
    .await
    .unwrap();
    assert!(stored.is_none());
}

/// `changed_at` defaults to `now()`, which is transaction-start time, so every
/// key touched by one patch carries the same timestamp — a tie is the normal
/// case here, not a rare one. The newer entry has to win on `id`.
#[tokio::test]
async fn entries_sharing_a_timestamp_are_separated_by_their_id() {
    let app = spawn_app().await;
    let org = child_org(&app, "Gleichzeitig", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    let older = Uuid::now_v7();
    let newer = Uuid::now_v7();
    let changed_at = chrono::Utc::now();

    for (id, liters) in [(older, 90.0), (newer, 120.0)] {
        sqlx::query!(
            r#"INSERT INTO organization_settings_history
                   (id, organization_id, setting_key, previous_value, new_value, changed_at)
               VALUES ($1, $2, $3, NULL, $4, $5)"#,
            id,
            org,
            SettingKey::WaterDemand.as_str(),
            serde_json::json!(liters),
            changed_at,
        )
        .execute(&app.db_pool)
        .await
        .unwrap();
    }

    let last = app
        .state
        .settings_service
        .last_changes(Id::<Organization>::new(org))
        .await
        .unwrap();

    let entry = last
        .get(&SettingKey::WaterDemand)
        .expect("entry for the changed key");
    assert_eq!(entry.next, Some(serde_json::json!(120.0)));
}

#[tokio::test]
async fn the_last_change_per_key_is_readable() {
    let app = spawn_app().await;
    let org = child_org(&app, "Letzte", Uuid::parse_str(ROOT_ORG_ID).unwrap()).await;

    set_demand(&app, org, 90.0).await;
    set_demand(&app, org, 120.0).await;

    let last = app
        .state
        .settings_service
        .last_changes(Id::<Organization>::new(org))
        .await
        .unwrap();

    let entry = last
        .get(&SettingKey::WaterDemand)
        .expect("entry for the changed key");
    assert_eq!(entry.next, Some(serde_json::json!(120.0)));
    assert_eq!(entry.previous, Some(serde_json::json!(90.0)));
}
