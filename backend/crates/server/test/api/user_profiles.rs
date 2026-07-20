use crate::helpers::spawn_app;
use domain::{
    user::{UserProfile, UserProfileReader, UserProfileWriter, UserStatus},
    vehicle::DrivingLicense,
};
use server::infra::pg_user_profile::PgUserProfileRepository;
use uuid::Uuid;

fn sample_profile(id: Uuid) -> UserProfile {
    UserProfile {
        id,
        employee_id: Some("EMP-42".into()),
        phone_number: Some("+49 461 123456".into()),
        avatar_url: Some(url::Url::parse("https://example.com/a.png").unwrap()),
        status: UserStatus::Absent,
        driving_licenses: vec![DrivingLicense::B, DrivingLicense::CE],
    }
}

#[tokio::test]
async fn upsert_and_read_back_roundtrip() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let id = Uuid::now_v7();

    repo.upsert(&sample_profile(id)).await.unwrap();

    let loaded = repo.by_ids(&[id]).await.unwrap();
    assert_eq!(loaded, vec![sample_profile(id)]);
}

#[tokio::test]
async fn upsert_replaces_existing_row() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let id = Uuid::now_v7();
    repo.upsert(&sample_profile(id)).await.unwrap();

    let replacement = UserProfile {
        status: UserStatus::Available,
        driving_licenses: vec![DrivingLicense::B],
        employee_id: None,
        ..sample_profile(id)
    };
    repo.upsert(&replacement).await.unwrap();

    let loaded = repo.by_ids(&[id]).await.unwrap();
    assert_eq!(loaded, vec![replacement]);
}

#[tokio::test]
async fn by_ids_skips_missing_rows() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let existing = Uuid::now_v7();
    repo.upsert(&sample_profile(existing)).await.unwrap();

    let loaded = repo.by_ids(&[existing, Uuid::now_v7()]).await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, existing);
}
