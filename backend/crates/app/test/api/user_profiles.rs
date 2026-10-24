use crate::helpers::spawn_app;
use domain::{
    Id,
    shared::phone_number::PhoneNumber,
    user::{UserProfile, UserProfileReader, UserProfileWriter, UserStatus},
    vehicle::DrivingLicense,
};
use server::infra::pg_user_profile::PgUserProfileRepository;
use uuid::Uuid;

const ROOT_ORG: &str = "01980000-0000-7000-8000-000000000001";

async fn seed_profile_row(repo: &PgUserProfileRepository, id: Uuid) {
    repo.set_organization(id, Id::new(Uuid::parse_str(ROOT_ORG).unwrap()))
        .await
        .unwrap();
}

fn sample_profile(id: Uuid) -> UserProfile {
    UserProfile {
        id,
        employee_id: Some("EMP-42".into()),
        phone_number: Some(PhoneNumber::new("+49 461 123456").unwrap()),
        avatar_url: Some(url::Url::parse("https://example.com/a.png").unwrap()),
        status: UserStatus::Absent,
        driving_licenses: vec![DrivingLicense::B, DrivingLicense::CE],
        watering_plan_selectable: false,
    }
}

#[tokio::test]
async fn upsert_and_read_back_roundtrip() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let id = Uuid::now_v7();

    seed_profile_row(&repo, id).await;
    repo.update(&sample_profile(id)).await.unwrap();

    let loaded = repo.by_ids(&[id]).await.unwrap();
    assert_eq!(loaded, vec![sample_profile(id)]);
}

#[tokio::test]
async fn upsert_replaces_existing_row() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let id = Uuid::now_v7();
    seed_profile_row(&repo, id).await;
    repo.update(&sample_profile(id)).await.unwrap();

    let replacement = UserProfile {
        status: UserStatus::Available,
        driving_licenses: vec![DrivingLicense::B],
        employee_id: None,
        ..sample_profile(id)
    };
    repo.update(&replacement).await.unwrap();

    let loaded = repo.by_ids(&[id]).await.unwrap();
    assert_eq!(loaded, vec![replacement]);
}

#[tokio::test]
async fn by_ids_skips_missing_rows() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let existing = Uuid::now_v7();
    seed_profile_row(&repo, existing).await;
    repo.update(&sample_profile(existing)).await.unwrap();

    let loaded = repo.by_ids(&[existing, Uuid::now_v7()]).await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, existing);
}

#[tokio::test]
async fn fresh_profile_row_is_selectable_for_watering_plans() {
    let app = spawn_app().await;
    let repo = PgUserProfileRepository::new(app.db_pool.clone());
    let id = Uuid::now_v7();

    seed_profile_row(&repo, id).await;

    let loaded = repo.by_ids(&[id]).await.unwrap();
    assert!(loaded[0].watering_plan_selectable);
}
