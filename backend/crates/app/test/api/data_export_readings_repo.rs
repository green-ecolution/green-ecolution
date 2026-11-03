use domain::{
    Id,
    data_export::{
        BASE_URL_FIELD, ExportAuth, ExportReadingSource, ExportRunStore, ExportTargetConfig,
        ExportTargetWriter, ExportUrl,
    },
    module::ModuleKey,
    sensor_model::SensorAbilityName,
};
use server::infra::pg_data_export::PgDataExportRepository;
use uuid::Uuid;

use crate::{
    helpers::{TestApp, spawn_app},
    organizations::ROOT_ORG_ID,
};

fn root() -> Uuid {
    Uuid::parse_str(ROOT_ORG_ID).unwrap()
}

/// A GES-1000 sensor on a fresh tree with one reading of two values.
async fn seed(app: &TestApp) -> (String, Uuid, Uuid) {
    let tree = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO trees (id, number, species, planting_year, latitude, longitude, watering_status, organization_id)
         VALUES ($1, 'R1', 'Linde', 2010, 54.79, 9.43, 'good', $2)",
    )
    .bind(tree)
    .bind(root())
    .execute(&app.db_pool)
    .await
    .unwrap();
    let sensor = format!("eui-{}", Uuid::now_v7().simple());
    sqlx::query(
        "INSERT INTO sensors (id, model_id, type, organization_id)
         SELECT $1, id, 'lorawan', $2 FROM sensor_models WHERE name = 'GES-1000'",
    )
    .bind(&sensor)
    .bind(root())
    .execute(&app.db_pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO sensor_attachments (sensor_id, module_key, target_kind, target_id) VALUES ($1, 'irrigation', 'tree', $2)")
        .bind(&sensor)
        .bind(tree)
        .execute(&app.db_pool)
        .await
        .unwrap();
    let reading = Uuid::now_v7();
    sqlx::query("INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, '{}')")
        .bind(reading)
        .bind(&sensor)
        .execute(&app.db_pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sensor_data_ability_values (sensor_data_id, sensor_model_ability_id, value, plausible, quality_reason)
         SELECT $1, ma.id, CASE WHEN ma.depth_cm = 40 THEN 42 ELSE 71 END, ma.depth_cm = 40, CASE WHEN ma.depth_cm = 40 THEN NULL ELSE 'out of range' END
         FROM sensor_model_abilities ma
         JOIN sensor_models m ON m.id = ma.sensor_model_id
         JOIN sensor_abilities a ON a.id = ma.sensor_ability_id
         WHERE m.name = 'GES-1000' AND a.ability = 'soil_moisture'",
    )
    .bind(reading)
    .execute(&app.db_pool)
    .await
    .unwrap();
    (sensor, tree, reading)
}

fn irrigation_root() -> Vec<(ModuleKey, Id<domain::organization::Organization>)> {
    vec![(ModuleKey::new("irrigation").unwrap(), Id::new(root()))]
}

#[tokio::test]
async fn readings_after_the_cursor_come_with_target_ability_and_quality() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let (sensor, tree, reading) = seed(&app).await;

    let rows = repo
        .readings_after(&irrigation_root(), Uuid::nil(), 10)
        .await
        .unwrap()
        .rows;
    let ours: Vec<_> = rows.iter().filter(|r| r.sensor_id == sensor).collect();

    assert_eq!(ours.len(), 2);
    assert!(
        ours.iter()
            .all(|r| r.reading_id == reading && r.target.id == tree)
    );
    assert!(
        ours.iter()
            .all(|r| r.ability == SensorAbilityName::SoilMoisture)
    );
    let deep = ours.iter().find(|r| r.depth_cm == 80).unwrap();
    assert_eq!(deep.value, 71.0);
    assert_eq!(deep.quality_issue.as_deref(), Some("out of range"));
    assert_eq!(
        ours.iter()
            .find(|r| r.depth_cm == 40)
            .unwrap()
            .quality_issue,
        None
    );

    assert!(
        repo.readings_after(&irrigation_root(), reading, 10)
            .await
            .unwrap()
            .rows
            .iter()
            .all(|r| r.sensor_id != sensor)
    );
    let waste_only = vec![(ModuleKey::new("waste").unwrap(), Id::new(root()))];
    assert!(
        repo.readings_after(&waste_only, Uuid::nil(), 10)
            .await
            .unwrap()
            .rows
            .iter()
            .all(|r| r.sensor_id != sensor)
    );
}

#[tokio::test]
async fn the_cursor_is_fenced_by_the_target_generation() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let cfg = |url: &str| ExportTargetConfig {
        enabled: true,
        base_url: ExportUrl::service_root(BASE_URL_FIELD, url).unwrap(),
        auth: ExportAuth::None,
    };
    let a = repo
        .upsert(Id::new(root()), &cfg("https://a.example.org/v1.1"))
        .await
        .unwrap();
    assert_eq!(repo.cursor(Id::new(root())).await.unwrap(), None);

    let c = Uuid::now_v7();
    repo.advance_cursor(Id::new(root()), a.target.activated_at, c)
        .await
        .unwrap();
    assert_eq!(repo.cursor(Id::new(root())).await.unwrap(), Some(c));

    repo.advance_cursor(Id::new(root()), a.target.activated_at, Uuid::nil())
        .await
        .unwrap();
    assert_eq!(
        repo.cursor(Id::new(root())).await.unwrap(),
        Some(c),
        "the cursor never moves backwards"
    );

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    repo.upsert(Id::new(root()), &cfg("https://b.example.org/v1.1"))
        .await
        .unwrap();
    repo.advance_cursor(Id::new(root()), a.target.activated_at, Uuid::now_v7())
        .await
        .unwrap();
    assert_eq!(
        repo.cursor(Id::new(root())).await.unwrap(),
        None,
        "stale pass must not move the new cursor"
    );
}

#[tokio::test]
async fn the_limit_counts_readings_and_value_less_readings_move_the_frontier() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let (sensor, _, first) = seed(&app).await;

    let limited = repo
        .readings_after(&irrigation_root(), first_before(first), 1)
        .await
        .unwrap();
    assert_eq!(limited.scanned, 1);
    assert_eq!(limited.rows.len(), 2);

    let second = Uuid::now_v7();
    sqlx::query("INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, '{}')")
        .bind(second)
        .bind(&sensor)
        .execute(&app.db_pool)
        .await
        .unwrap();
    let batch = repo
        .readings_after(&irrigation_root(), first, 10)
        .await
        .unwrap();
    assert!(batch.rows.iter().all(|r| r.sensor_id != sensor));
    assert_eq!(batch.scanned_until, Some(second));
    assert_eq!(batch.scanned, 1);
}

/// Seed data may hold older readings; start just below the fresh one.
fn first_before(reading: Uuid) -> Uuid {
    Uuid::from_u128(reading.as_u128() - 1)
}
