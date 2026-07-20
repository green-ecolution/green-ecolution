use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use domain::{
    RepositoryError,
    shared::{
        email::Email,
        pagination::{Page, Pagination},
    },
    user::{
        UserCreate, UserIdentity, UserProfile, UserProfileReader, UserProfileWriter,
        UserRepository, UserRole, UserStatus, UserView, Username,
    },
    vehicle::DrivingLicense,
};

use super::ServiceError;

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    profile_reader: Arc<dyn UserProfileReader>,
    profile_writer: Arc<dyn UserProfileWriter>,
    enabled: bool,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        profile_reader: Arc<dyn UserProfileReader>,
        profile_writer: Arc<dyn UserProfileWriter>,
        enabled: bool,
    ) -> Self {
        Self {
            user_repo,
            profile_reader,
            profile_writer,
            enabled,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn register(&self, entity: UserCreate) -> Result<UserView, ServiceError> {
        if !self.enabled {
            return Ok(synthesize_registered_user(entity));
        }
        let profile_fields = (
            entity.employee_id.clone(),
            entity.phone_number.clone(),
            entity.avatar_url.clone(),
            entity.status,
            entity.driving_licenses.clone(),
        );
        let identity = self.user_repo.create(entity).await?;
        let profile = UserProfile {
            id: identity.id,
            employee_id: profile_fields.0,
            phone_number: profile_fields.1,
            avatar_url: profile_fields.2,
            status: profile_fields.3,
            driving_licenses: profile_fields.4,
        };
        self.profile_writer.upsert(&profile).await?;
        Ok(merge(identity, Some(profile)))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self, pagination: Pagination) -> Result<Page<UserView>, ServiceError> {
        if !self.enabled {
            return Ok(demo_user_page(pagination));
        }
        let page = self.user_repo.all(pagination).await?;
        let items = self.attach_profiles(page.items).await?;
        Ok(Page {
            items,
            total: page.total,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_role(
        &self,
        role: UserRole,
        pagination: Pagination,
    ) -> Result<Page<UserView>, ServiceError> {
        if !self.enabled {
            if role == UserRole::GreenEcolution {
                return Ok(demo_user_page(pagination));
            }
            return Ok(Page {
                items: Vec::new(),
                total: 0,
            });
        }
        let page = self.user_repo.by_role(role, pagination).await?;
        let items = self.attach_profiles(page.items).await?;
        Ok(Page {
            items,
            total: page.total,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserView>, ServiceError> {
        if !self.enabled {
            let demo = demo_user();
            return Ok(ids
                .iter()
                .filter(|id| **id == demo.id)
                .map(|_| demo.clone())
                .collect());
        }
        let identities = self.user_repo.by_ids(ids).await?;
        self.attach_profiles(identities).await
    }

    /// In demo mode (auth disabled) the write is a no-op: the static demo user is returned unchanged.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_profile(&self, profile: UserProfile) -> Result<UserView, ServiceError> {
        if !self.enabled {
            let demo = demo_user();
            if profile.id == demo.id {
                return Ok(demo);
            }
            return Err(RepositoryError::NotFound.into());
        }
        let identity = self
            .user_repo
            .by_ids(&[profile.id])
            .await?
            .into_iter()
            .next()
            .ok_or(RepositoryError::NotFound)?;
        self.profile_writer.upsert(&profile).await?;
        Ok(merge(identity, Some(profile)))
    }

    async fn attach_profiles(
        &self,
        identities: Vec<UserIdentity>,
    ) -> Result<Vec<UserView>, ServiceError> {
        if identities.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = identities.iter().map(|i| i.id).collect();
        let mut profiles: HashMap<Uuid, UserProfile> = self
            .profile_reader
            .by_ids(&ids)
            .await?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();
        Ok(identities
            .into_iter()
            .map(|identity| {
                let profile = profiles.remove(&identity.id);
                merge(identity, profile)
            })
            .collect())
    }
}

fn merge(identity: UserIdentity, profile: Option<UserProfile>) -> UserView {
    let profile = profile.unwrap_or_else(|| UserProfile::empty(identity.id));
    UserView {
        id: identity.id,
        created_at: identity.created_at,
        username: identity.username,
        first_name: identity.first_name,
        last_name: identity.last_name,
        email: identity.email,
        email_verified: identity.email_verified,
        employee_id: profile.employee_id,
        phone_number: profile.phone_number,
        avatar_url: profile.avatar_url,
        roles: identity.roles,
        driving_licenses: profile.driving_licenses,
        status: profile.status,
    }
}

// Must match the anonymous demo user injected by auth_middleware when auth.enabled = false.
fn demo_user() -> UserView {
    UserView {
        id: Uuid::nil(),
        created_at: Utc::now(),
        username: Username::reconstitute("ttester".to_string()),
        first_name: "Toni".into(),
        last_name: "Tester".into(),
        email: Email::reconstitute("toni.tester@green-ecolution.de".to_string()),
        email_verified: true,
        employee_id: None,
        phone_number: None,
        avatar_url: None,
        roles: vec![UserRole::GreenEcolution],
        driving_licenses: vec![
            DrivingLicense::B,
            DrivingLicense::BE,
            DrivingLicense::C,
            DrivingLicense::CE,
        ],
        status: UserStatus::Available,
    }
}

fn demo_user_page(pagination: Pagination) -> Page<UserView> {
    let items = if pagination.page() == 1 {
        vec![demo_user()]
    } else {
        Vec::new()
    };
    Page { items, total: 1 }
}

fn synthesize_registered_user(entity: UserCreate) -> UserView {
    let roles = if entity.roles.is_empty() {
        vec![UserRole::GreenEcolution]
    } else {
        entity.roles.clone()
    };
    UserView {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        username: entity.username,
        first_name: entity.first_name,
        last_name: entity.last_name,
        email: entity.email,
        email_verified: false,
        employee_id: entity.employee_id,
        phone_number: entity.phone_number,
        avatar_url: entity.avatar_url,
        roles,
        driving_licenses: entity.driving_licenses,
        status: entity.status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        RepositoryError,
        user::{UserIdentity, UserProfile, UserProfileReader, UserProfileWriter},
    };
    use secrecy::SecretString;
    use std::{collections::HashMap, sync::Mutex};

    struct StubIdentityRepo {
        identities: Vec<UserIdentity>,
    }

    #[async_trait::async_trait]
    impl UserRepository for StubIdentityRepo {
        async fn create(&self, entity: UserCreate) -> Result<UserIdentity, RepositoryError> {
            Ok(identity(Uuid::now_v7(), entity.username.as_str()))
        }
        async fn all(&self, _p: Pagination) -> Result<Page<UserIdentity>, RepositoryError> {
            Ok(Page {
                items: self.identities.clone(),
                total: self.identities.len() as u64,
            })
        }
        async fn by_role(
            &self,
            _r: UserRole,
            _p: Pagination,
        ) -> Result<Page<UserIdentity>, RepositoryError> {
            self.all(_p).await
        }
        async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserIdentity>, RepositoryError> {
            Ok(self
                .identities
                .iter()
                .filter(|i| ids.contains(&i.id))
                .cloned()
                .collect())
        }
    }

    #[derive(Default)]
    struct InMemoryProfiles {
        rows: Mutex<HashMap<Uuid, UserProfile>>,
    }

    #[async_trait::async_trait]
    impl UserProfileReader for InMemoryProfiles {
        async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError> {
            let rows = self.rows.lock().unwrap();
            Ok(ids.iter().filter_map(|id| rows.get(id).cloned()).collect())
        }
    }

    #[async_trait::async_trait]
    impl UserProfileWriter for InMemoryProfiles {
        async fn upsert(&self, profile: &UserProfile) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .insert(profile.id, profile.clone());
            Ok(())
        }
    }

    fn identity(id: Uuid, username: &str) -> UserIdentity {
        UserIdentity {
            id,
            created_at: Utc::now(),
            username: Username::reconstitute(username.to_string()),
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            email: Email::reconstitute(format!("{username}@example.com")),
            email_verified: true,
            roles: vec![UserRole::Tbz],
        }
    }

    fn service(identities: Vec<UserIdentity>, profiles: Arc<InMemoryProfiles>) -> UserService {
        UserService::new(
            Arc::new(StubIdentityRepo { identities }),
            profiles.clone(),
            profiles,
            true,
        )
    }

    #[tokio::test]
    async fn list_merges_profile_into_view() {
        let id = Uuid::now_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles
            .upsert(&UserProfile {
                id,
                employee_id: Some("EMP-1".into()),
                phone_number: None,
                avatar_url: None,
                status: UserStatus::Absent,
                driving_licenses: vec![DrivingLicense::CE],
            })
            .await
            .unwrap();
        let svc = service(vec![identity(id, "jdoe")], profiles);

        let page = svc.list(Pagination::default()).await.unwrap();

        assert_eq!(page.items[0].employee_id.as_deref(), Some("EMP-1"));
        assert_eq!(page.items[0].status, UserStatus::Absent);
        assert_eq!(page.items[0].driving_licenses, vec![DrivingLicense::CE]);
        assert_eq!(page.items[0].username.as_str(), "jdoe");
    }

    #[tokio::test]
    async fn missing_profile_yields_defaults() {
        let id = Uuid::now_v7();
        let svc = service(
            vec![identity(id, "jdoe")],
            Arc::new(InMemoryProfiles::default()),
        );

        let page = svc.list(Pagination::default()).await.unwrap();

        assert_eq!(page.items[0].status, UserStatus::Available);
        assert!(page.items[0].driving_licenses.is_empty());
        assert!(page.items[0].employee_id.is_none());
    }

    #[tokio::test]
    async fn register_upserts_profile() {
        let profiles = Arc::new(InMemoryProfiles::default());
        let svc = service(vec![], profiles.clone());
        let entity = UserCreate {
            username: Username::reconstitute("new".into()),
            first_name: "New".into(),
            last_name: "User".into(),
            email: Email::reconstitute("new@example.com".into()),
            password: SecretString::from("pw".to_string()),
            roles: vec![],
            employee_id: Some("EMP-9".into()),
            phone_number: None,
            avatar_url: None,
            status: UserStatus::Absent,
            driving_licenses: vec![DrivingLicense::B],
        };

        let view = svc.register(entity).await.unwrap();

        let stored = profiles.by_ids(&[view.id]).await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].employee_id.as_deref(), Some("EMP-9"));
        assert_eq!(view.status, UserStatus::Absent);
    }

    #[tokio::test]
    async fn update_profile_returns_not_found_for_unknown_identity() {
        let svc = service(vec![], Arc::new(InMemoryProfiles::default()));

        let result = svc.update_profile(UserProfile::empty(Uuid::now_v7())).await;

        assert!(matches!(
            result,
            Err(ServiceError::Repository(RepositoryError::NotFound))
        ));
    }

    #[tokio::test]
    async fn update_profile_upserts_and_returns_merged_view() {
        let id = Uuid::now_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        let svc = service(vec![identity(id, "jdoe")], profiles.clone());
        let mut profile = UserProfile::empty(id);
        profile.status = UserStatus::Absent;

        let view = svc.update_profile(profile).await.unwrap();

        assert_eq!(view.status, UserStatus::Absent);
        assert_eq!(profiles.by_ids(&[id]).await.unwrap().len(), 1);
    }
}
