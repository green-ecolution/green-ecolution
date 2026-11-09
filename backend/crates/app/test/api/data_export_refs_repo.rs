use domain::{
    Id,
    data_export::{
        BASE_URL_FIELD, ExportAuth, ExportEntity, ExportLease, ExportRef, ExportRefReader,
        ExportRefWriter, ExportRunStats, ExportRunStore, ExportTargetConfig, ExportTargetWriter,
        ExportUrl, RemoteId, RunResult,
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

/// Leases the root target under a fresh token, as a pass does.
async fn lease(repo: &PgDataExportRepository, interval_secs: i64) -> Option<ExportLease> {
    let token = Uuid::now_v7();
    repo.try_lease(root(), token, 60, interval_secs)
        .await
        .unwrap()
        .map(|leased| ExportLease {
            generation: leased.target.activated_at,
            token,
        })
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
    let lease = lease(&repo, 0).await.unwrap();

    repo.put_ref(root(), &lease, &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    let mut changed = thing_ref("irrigation:tree:1");
    changed.content_hash = Some("def".into());
    changed.deleted_marked = true;
    repo.put_ref(root(), &lease, &changed).await.unwrap();

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
    let lease = lease(&repo, 0).await.unwrap();
    repo.put_ref(root(), &lease, &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    repo.finish(
        root(),
        &lease,
        &RunResult::Succeeded(ExportRunStats::default()),
    )
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

    assert_eq!(repo.due(0).await.unwrap(), vec![root()]);
    let held = lease(&repo, 0).await.unwrap();
    assert!(lease(&repo, 0).await.is_none());
    assert!(repo.due(0).await.unwrap().is_empty());

    repo.finish(
        root(),
        &held,
        &RunResult::Succeeded(ExportRunStats {
            things_written: 2,
            observations_written: 3,
            skipped: 0,
        }),
    )
    .await
    .unwrap();
    let leased = repo
        .try_lease(root(), Uuid::now_v7(), 60, 0)
        .await
        .unwrap()
        .unwrap();
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
    let first = lease(&repo, 0).await.unwrap();

    repo.finish(
        root(),
        &first,
        &RunResult::Failed {
            error: "target unreachable".into(),
            base_interval_secs: 300,
        },
    )
    .await
    .unwrap();

    assert!(repo.due(0).await.unwrap().is_empty(), "still backing off");
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
    let second = lease(&repo, 0).await.unwrap();
    repo.finish(
        root(),
        &second,
        &RunResult::Succeeded(ExportRunStats::default()),
    )
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
    assert!(repo.due(0).await.unwrap().is_empty());
}

#[tokio::test]
async fn the_backoff_is_capped_after_many_failures() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    sqlx::query("UPDATE data_export_targets SET failure_count = 2000")
        .execute(&app.db_pool)
        .await
        .unwrap();
    let held = lease(&repo, 0).await.unwrap();

    repo.finish(
        root(),
        &held,
        &RunResult::Failed {
            error: "down".into(),
            base_interval_secs: 300,
        },
    )
    .await
    .unwrap();

    let (count, wait): (i32, f64) = sqlx::query_as(
        "SELECT failure_count, EXTRACT(EPOCH FROM next_attempt_at - now())::float8 FROM data_export_targets",
    )
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(count, 2001);
    assert!(
        (3590.0..=3600.0).contains(&wait),
        "capped at 3600 s, got {wait}"
    );
}

#[tokio::test]
async fn a_pass_from_before_a_base_url_change_writes_nothing() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    let stale = lease(&repo, 0).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    repo.upsert(root(), &config("https://b.example.org/v1.1"))
        .await
        .unwrap();

    repo.put_ref(root(), &stale, &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    repo.finish(
        root(),
        &stale,
        &RunResult::Succeeded(ExportRunStats::default()),
    )
    .await
    .unwrap();

    assert!(repo.refs(root()).await.unwrap().is_empty());
    let success: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_success_at FROM data_export_targets")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(success, None);
}

#[tokio::test]
async fn a_finished_run_is_not_leased_again_within_the_interval() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    let held = lease(&repo, 300).await.unwrap();
    repo.finish(
        root(),
        &held,
        &RunResult::Succeeded(ExportRunStats::default()),
    )
    .await
    .unwrap();

    assert!(lease(&repo, 300).await.is_none());
    assert!(lease(&repo, 300).await.is_none());
    assert!(repo.due(300).await.unwrap().is_empty());

    sqlx::query("UPDATE data_export_targets SET last_run_at = now() - interval '301 seconds'")
        .execute(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(repo.due(300).await.unwrap(), vec![root()]);
    assert!(lease(&repo, 300).await.is_some());
}

#[tokio::test]
async fn saving_the_target_lifts_the_backoff_and_the_error() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let gen_ = repo
        .upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap()
        .target
        .activated_at;
    let held = lease(&repo, 0).await.unwrap();
    repo.put_ref(root(), &held, &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    repo.finish(
        root(),
        &held,
        &RunResult::Failed {
            error: "target refused the credentials (HTTP 401)".into(),
            base_interval_secs: 300,
        },
    )
    .await
    .unwrap();
    assert!(repo.due(0).await.unwrap().is_empty(), "backing off");

    let saved = repo
        .upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();

    assert_eq!(saved.status.last_error, None);
    assert!(saved.status.last_run_at.is_some(), "the last run stays");
    assert_eq!(saved.target.activated_at, gen_);
    let (count, next): (i32, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT failure_count, next_attempt_at FROM data_export_targets")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!((count, next), (0, None));
    assert_eq!(repo.due(0).await.unwrap(), vec![root()]);
    assert_eq!(repo.refs(root()).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_ref_written_during_a_generation_change_waits_and_is_dropped() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    let stale = lease(&repo, 0).await.unwrap();
    let mut change = app.db_pool.begin().await.unwrap();
    sqlx::query("UPDATE data_export_targets SET activated_at = activated_at + interval '1 second'")
        .execute(&mut *change)
        .await
        .unwrap();

    let writer = PgDataExportRepository::new(app.db_pool.clone());
    let put = tokio::spawn(async move {
        writer
            .put_ref(root(), &stale, &thing_ref("irrigation:tree:1"))
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    change.commit().await.unwrap();
    put.await.unwrap().unwrap();

    assert!(repo.refs(root()).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_pass_whose_lease_was_taken_over_writes_nothing() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    let old = lease(&repo, 0).await.unwrap();
    sqlx::query("UPDATE data_export_targets SET leased_until = now() - interval '1 second'")
        .execute(&app.db_pool)
        .await
        .unwrap();
    let new = lease(&repo, 0)
        .await
        .expect("an expired lease can be taken over");

    assert!(!repo.renew_lease(root(), &old, 60).await.unwrap());
    repo.put_ref(root(), &old, &thing_ref("irrigation:tree:1"))
        .await
        .unwrap();
    repo.advance_cursor(root(), &old, Uuid::now_v7())
        .await
        .unwrap();
    repo.finish(
        root(),
        &old,
        &RunResult::Succeeded(ExportRunStats::default()),
    )
    .await
    .unwrap();

    assert!(repo.refs(root()).await.unwrap().is_empty());
    assert_eq!(repo.cursor(root()).await.unwrap(), None);
    assert!(
        lease(&repo, 0).await.is_none(),
        "the old pass must not end the new pass's lease"
    );
    assert!(repo.renew_lease(root(), &new, 60).await.unwrap());
}
