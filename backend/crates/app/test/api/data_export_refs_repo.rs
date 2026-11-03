use domain::{
    Id,
    data_export::{
        BASE_URL_FIELD, ExportAuth, ExportEntity, ExportRef, ExportRefReader, ExportRefWriter,
        ExportRunStats, ExportRunStore, ExportTargetConfig, ExportTargetWriter, ExportUrl,
        RemoteId, RunResult,
    },
};
use serde_json::json;
use server::infra::pg_data_export::PgDataExportRepository;
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

fn root() -> Id<domain::organization::Organization> {
    Id::new(Uuid::parse_str(ROOT_ORG_ID).unwrap())
}

fn config(url: &str) -> ExportTargetConfig {
    ExportTargetConfig {
        enabled: true,
        base_url: ExportUrl::service_root(BASE_URL_FIELD, url).unwrap(),
        auth: ExportAuth::None,
    }
}

fn thing_ref(key: &str) -> ExportRef {
    ExportRef {
        ge_key: key.into(),
        entity: ExportEntity::Thing,
        remote_id: RemoteId(json!(7)),
        content_hash: Some("abc".into()),
        last_value: None,
        deleted_marked: false,
    }
}

#[tokio::test]
async fn refs_are_stored_and_overwritten_by_key() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://frost.example.org/v1.1"))
        .await
        .unwrap();

    repo.put_ref(root(), &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    let mut changed = thing_ref("irrigation:tree:1");
    changed.content_hash = Some("def".into());
    changed.deleted_marked = true;
    repo.put_ref(root(), &changed).await.unwrap();

    let refs = repo.refs(root()).await.unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs["irrigation:tree:1"], changed);
}

#[tokio::test]
async fn a_new_base_url_drops_the_refs_and_the_last_run() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    repo.put_ref(root(), &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    repo.finish(root(), &RunResult::Succeeded(ExportRunStats::default()))
        .await
        .unwrap();

    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    assert_eq!(repo.refs(root()).await.unwrap().len(), 1);

    let moved = repo
        .upsert(root(), &config("https://b.example.org/v1.1"))
        .await
        .unwrap();
    assert!(repo.refs(root()).await.unwrap().is_empty());
    assert_eq!(moved.status.last_run_at, None);
}

#[tokio::test]
async fn a_lease_is_exclusive_until_the_run_finishes() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();

    assert_eq!(repo.due().await.unwrap(), vec![root()]);
    assert!(repo.try_lease(root(), 60).await.unwrap().is_some());
    assert!(repo.try_lease(root(), 60).await.unwrap().is_none());
    assert!(repo.due().await.unwrap().is_empty());

    repo.finish(
        root(),
        &RunResult::Succeeded(ExportRunStats {
            things_written: 2,
            observations_written: 3,
            skipped: 0,
        }),
    )
    .await
    .unwrap();
    let leased = repo.try_lease(root(), 60).await.unwrap().unwrap();
    assert_eq!(leased.status.last_run_stats.unwrap().things_written, 2);
    assert!(leased.status.last_success_at.is_some());
}

#[tokio::test]
async fn a_failure_backs_off_and_a_success_clears_it() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    repo.try_lease(root(), 60).await.unwrap().unwrap();

    repo.finish(
        root(),
        &RunResult::Failed {
            error: "target unreachable".into(),
            base_interval_secs: 300,
        },
    )
    .await
    .unwrap();

    assert!(repo.due().await.unwrap().is_empty(), "still backing off");
    let (count, wait): (i32, f64) = sqlx::query_as(
        "SELECT failure_count, EXTRACT(EPOCH FROM next_attempt_at - now())::float8 FROM data_export_targets",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
    assert!((590.0..=600.0).contains(&wait), "300 s × 2^1, got {wait}");

    sqlx::query("UPDATE data_export_targets SET next_attempt_at = now() - interval '1 second'")
        .execute(&app.db_pool)
        .await
        .unwrap();
    repo.try_lease(root(), 60).await.unwrap().unwrap();
    repo.finish(root(), &RunResult::Succeeded(ExportRunStats::default()))
        .await
        .unwrap();
    let (count, next): (i32, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT failure_count, next_attempt_at FROM data_export_targets")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!((count, next), (0, None));
}

#[tokio::test]
async fn target_orgs_include_disabled_targets() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let mut cfg = config("https://a.example.org/v1.1");
    cfg.enabled = false;
    repo.upsert(root(), &cfg).await.unwrap();

    assert!(repo.target_orgs().await.unwrap().contains(&root()));
    assert!(repo.due().await.unwrap().is_empty());
}
