use domain::{
    Id,
    data_export::{
        BASE_URL_FIELD, ExportAuth, ExportPrincipal, ExportSecret, ExportTargetConfig,
        ExportTargetReader, ExportTargetWriter, ExportUrl,
    },
};
use secrecy::SecretString;
use server::infra::pg_data_export::PgDataExportRepository;
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

fn config(url: &str) -> ExportTargetConfig {
    ExportTargetConfig {
        enabled: true,
        base_url: ExportUrl::new(BASE_URL_FIELD, url).unwrap(),
        auth: ExportAuth::Basic {
            username: ExportPrincipal::new("ge").unwrap(),
            password: ExportSecret::new("f", SecretString::from("pw".to_string())).unwrap(),
        },
    }
}

fn root() -> Id<domain::organization::Organization> {
    Id::new(Uuid::parse_str(ROOT_ORG_ID).unwrap())
}

#[tokio::test]
async fn upsert_creates_and_find_reads_back() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());

    let saved = repo
        .upsert(root(), &config("https://frost.example.org/v1.1"))
        .await
        .unwrap();
    let found = repo.find(root()).await.unwrap().unwrap();

    assert_eq!(
        found.target.config.base_url.as_str(),
        "https://frost.example.org/v1.1"
    );
    assert_eq!(found.target.config.auth.secret().unwrap().expose(), "pw");
    assert_eq!(found.target.activated_at, saved.target.activated_at);
    assert_eq!(found.status, Default::default());
}

#[tokio::test]
async fn a_new_base_url_restarts_activation_and_cursor() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    let first = repo
        .upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    sqlx::query!(
        "UPDATE data_export_targets SET cursor_reading_id = $1, activated_at = now() - interval '1 day'",
        Uuid::now_v7()
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let same = repo
        .upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();
    assert!(same.target.activated_at < first.target.activated_at);

    let moved = repo
        .upsert(root(), &config("https://b.example.org/v1.1"))
        .await
        .unwrap();
    assert!(moved.target.activated_at > same.target.activated_at);
    let cursor: Option<Uuid> =
        sqlx::query_scalar!("SELECT cursor_reading_id FROM data_export_targets")
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(cursor, None);
}

#[tokio::test]
async fn an_unknown_organization_is_not_found() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());

    let err = repo
        .upsert(
            Id::new(Uuid::now_v7()),
            &config("https://a.example.org/v1.1"),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, domain::RepositoryError::NotFound));
}

#[tokio::test]
async fn delete_removes_and_reports_a_missing_target() {
    let app = spawn_app().await;
    let repo = PgDataExportRepository::new(app.db_pool.clone());
    repo.upsert(root(), &config("https://a.example.org/v1.1"))
        .await
        .unwrap();

    repo.delete(root()).await.unwrap();
    assert!(repo.find(root()).await.unwrap().is_none());
    assert!(matches!(
        repo.delete(root()).await,
        Err(domain::RepositoryError::NotFound)
    ));
}
