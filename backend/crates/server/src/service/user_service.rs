use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    organization::{Organization, OrganizationReader, OrganizationView},
    role::{Role, RoleError, RoleReader, RoleView, RoleWriter},
    shared::{
        email::Email,
        pagination::{Page, Pagination},
    },
    user::{
        UserCreate, UserIdentity, UserIdentityCreate, UserProfile, UserProfileReader,
        UserProfileWriter, UserRepository, UserStatus, UserView, Username,
    },
    vehicle::DrivingLicense,
};

use super::ServiceError;

/// Optional list filters resolved against the local database (organization
/// membership and role assignments live in Postgres, not the IdP).
#[derive(Debug, Clone, Default)]
pub struct UserListFilter {
    pub organization_id: Option<Id<Organization>>,
    pub role_id: Option<Id<Role>>,
}

impl UserListFilter {
    fn is_empty(&self) -> bool {
        self.organization_id.is_none() && self.role_id.is_none()
    }
}

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    profile_reader: Arc<dyn UserProfileReader>,
    profile_writer: Arc<dyn UserProfileWriter>,
    role_reader: Arc<dyn RoleReader>,
    role_writer: Arc<dyn RoleWriter>,
    org_reader: Arc<dyn OrganizationReader>,
    enabled: bool,
}

impl UserService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        profile_reader: Arc<dyn UserProfileReader>,
        profile_writer: Arc<dyn UserProfileWriter>,
        role_reader: Arc<dyn RoleReader>,
        role_writer: Arc<dyn RoleWriter>,
        org_reader: Arc<dyn OrganizationReader>,
        enabled: bool,
    ) -> Self {
        Self {
            user_repo,
            profile_reader,
            profile_writer,
            role_reader,
            role_writer,
            org_reader,
            enabled,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn register(&self, entity: UserCreate) -> Result<UserView, ServiceError> {
        if !self.enabled {
            return Ok(synthesize_registered_user(entity));
        }
        let identity = self
            .user_repo
            .create(UserIdentityCreate {
                username: entity.username.clone(),
                first_name: entity.first_name.clone(),
                last_name: entity.last_name.clone(),
                email: entity.email.clone(),
                password: entity.password.clone(),
            })
            .await?;
        let profile = UserProfile {
            id: identity.id,
            employee_id: entity.employee_id,
            phone_number: entity.phone_number,
            avatar_url: entity.avatar_url,
            status: entity.status,
            driving_licenses: entity.driving_licenses,
        };
        self.profile_writer.upsert(&profile).await?;
        self.profile_writer
            .set_organization(identity.id, entity.organization_id)
            .await?;
        let mut assigned = Vec::new();
        for role_id in &entity.role_ids {
            let role = self.role_reader.by_id(*role_id).await?;
            if role.is_template() {
                return Err(RoleError::CannotAssignTemplate.into());
            }
            self.role_writer
                .assign_to_user(identity.id, *role_id)
                .await?;
            assigned.push(RoleView::from(&role));
        }
        let organization = Some((&self.org_reader.by_id(entity.organization_id).await?).into());
        Ok(merge(identity, Some(profile), organization, assigned))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(
        &self,
        pagination: Pagination,
        filter: UserListFilter,
    ) -> Result<Page<UserView>, ServiceError> {
        if filter.is_empty() {
            if !self.enabled {
                return Ok(demo_user_page(pagination));
            }
            let page = self.user_repo.all(pagination).await?;
            let items = self.attach_views(page.items).await?;
            return Ok(Page {
                items,
                total: page.total,
            });
        }

        let ids = self.filtered_ids(&filter).await?;
        let total = ids.len() as u64;
        let start = pagination.offset() as usize;
        let end = start
            .saturating_add(pagination.limit() as usize)
            .min(ids.len());
        let slice = ids.get(start..end).unwrap_or(&[]);
        let identities = self.identities_for(slice).await?;
        let items = self.attach_views(identities).await?;
        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_organization(
        &self,
        user_id: Uuid,
        org: Id<Organization>,
    ) -> Result<UserView, ServiceError> {
        self.profile_writer.set_organization(user_id, org).await?;
        let identity = self.identity_for(user_id).await?;
        self.attach_views(vec![identity])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::NotFound.into())
    }

    /// The target user's organization, if any. Returns `None` for legacy users
    /// created before organization membership existed.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn organization_of(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Id<Organization>>, ServiceError> {
        Ok(self
            .profile_reader
            .organizations_for(&[user_id])
            .await?
            .into_iter()
            .next()
            .map(|(_, org)| org))
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
        self.attach_views(identities).await
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
        self.attach_views(vec![identity])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::NotFound.into())
    }

    async fn filtered_ids(&self, filter: &UserListFilter) -> Result<Vec<Uuid>, ServiceError> {
        let by_org = match filter.organization_id {
            Some(org) => Some(self.profile_reader.ids_in_organization(org).await?),
            None => None,
        };
        let by_role = match filter.role_id {
            Some(role) => Some(self.role_reader.user_ids_with_role(role).await?),
            None => None,
        };
        Ok(match (by_org, by_role) {
            (Some(a), Some(b)) => {
                let keep: HashSet<Uuid> = b.into_iter().collect();
                a.into_iter().filter(|id| keep.contains(id)).collect()
            }
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => Vec::new(),
        })
    }

    /// Resolve identities for a page of ids. In demo mode only the static demo
    /// user is resolvable, so seeded/foreign ids yield no identities.
    async fn identities_for(&self, ids: &[Uuid]) -> Result<Vec<UserIdentity>, ServiceError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        if !self.enabled {
            let demo = demo_user();
            return Ok(ids
                .iter()
                .filter(|id| **id == demo.id)
                .map(|id| demo_identity(*id))
                .collect());
        }
        Ok(self.user_repo.by_ids(ids).await?)
    }

    async fn identity_for(&self, user_id: Uuid) -> Result<UserIdentity, ServiceError> {
        if !self.enabled {
            return Ok(demo_identity(user_id));
        }
        self.user_repo
            .by_ids(&[user_id])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::NotFound.into())
    }

    async fn attach_views(
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
        let mut roles: HashMap<Uuid, Vec<RoleView>> = HashMap::new();
        for (user_id, role) in self.role_reader.roles_for_users(&ids).await? {
            roles
                .entry(user_id)
                .or_default()
                .push(RoleView::from(&role));
        }
        let org_ids: HashMap<Uuid, Id<Organization>> = self
            .profile_reader
            .organizations_for(&ids)
            .await?
            .into_iter()
            .collect();
        let mut orgs: HashMap<Id<Organization>, OrganizationView> = HashMap::new();
        for org_id in org_ids.values() {
            if !orgs.contains_key(org_id) {
                orgs.insert(*org_id, (&self.org_reader.by_id(*org_id).await?).into());
            }
        }
        Ok(identities
            .into_iter()
            .map(|identity| {
                let profile = profiles.remove(&identity.id);
                let organization = org_ids.get(&identity.id).and_then(|o| orgs.get(o)).cloned();
                let user_roles = roles.remove(&identity.id).unwrap_or_default();
                merge(identity, profile, organization, user_roles)
            })
            .collect())
    }
}

fn merge(
    identity: UserIdentity,
    profile: Option<UserProfile>,
    organization: Option<OrganizationView>,
    roles: Vec<RoleView>,
) -> UserView {
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
        organization,
        roles,
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
        organization: None,
        roles: Vec::new(),
        driving_licenses: vec![
            DrivingLicense::B,
            DrivingLicense::BE,
            DrivingLicense::C,
            DrivingLicense::CE,
        ],
        status: UserStatus::Available,
    }
}

/// Placeholder identity used only in demo mode, where the IdP is not queried.
/// Organization and roles are still enriched from the local database.
fn demo_identity(id: Uuid) -> UserIdentity {
    UserIdentity {
        id,
        created_at: Utc::now(),
        username: Username::reconstitute("ttester".to_string()),
        first_name: "Toni".into(),
        last_name: "Tester".into(),
        email: Email::reconstitute("toni.tester@green-ecolution.de".to_string()),
        email_verified: true,
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
        organization: None,
        roles: Vec::new(),
        driving_licenses: entity.driving_licenses,
        status: entity.status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        Id, RepositoryError,
        authorization::OrgHierarchy,
        organization::{Organization, OrganizationSnapshot},
        role::{Role, RoleDraft},
        user::{UserIdentity, UserProfile, UserProfileReader, UserProfileWriter},
    };
    use secrecy::SecretString;
    use std::{collections::HashMap, sync::Mutex};

    struct StubIdentityRepo {
        identities: Vec<UserIdentity>,
    }

    #[async_trait::async_trait]
    impl UserRepository for StubIdentityRepo {
        async fn create(
            &self,
            entity: UserIdentityCreate,
        ) -> Result<UserIdentity, RepositoryError> {
            Ok(identity(Uuid::now_v7(), entity.username.as_str()))
        }
        async fn all(&self, _p: Pagination) -> Result<Page<UserIdentity>, RepositoryError> {
            Ok(Page {
                items: self.identities.clone(),
                total: self.identities.len() as u64,
            })
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
        async fn ids_in_organization(
            &self,
            _org: Id<Organization>,
        ) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn organizations_for(
            &self,
            _ids: &[Uuid],
        ) -> Result<Vec<(Uuid, Id<Organization>)>, RepositoryError> {
            Ok(Vec::new())
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
        async fn ensure_exists(&self, id: Uuid) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .entry(id)
                .or_insert_with(|| UserProfile::empty(id));
            Ok(())
        }
        async fn set_organization(
            &self,
            _id: Uuid,
            _org: Id<Organization>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct StubRoles;

    #[async_trait::async_trait]
    impl RoleReader for StubRoles {
        async fn by_id(&self, _id: Id<Role>) -> Result<Role, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn by_organization(
            &self,
            _org: Id<Organization>,
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn roles_for_user(&self, _user_id: Uuid) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn roles_for_users(
            &self,
            _ids: &[Uuid],
        ) -> Result<Vec<(Uuid, Role)>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn user_ids_with_role(
            &self,
            _role_id: Id<Role>,
        ) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }
    }

    #[async_trait::async_trait]
    impl RoleWriter for StubRoles {
        async fn save_new(&self, _draft: RoleDraft) -> Result<Role, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn save(&self, _role: &Role) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn delete(&self, _id: Id<Role>) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn assign_to_user(
            &self,
            _user_id: Uuid,
            _role_id: Id<Role>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn revoke_from_user(
            &self,
            _user_id: Uuid,
            _role_id: Id<Role>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct StubOrgs;

    #[async_trait::async_trait]
    impl OrganizationReader for StubOrgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Ok(Organization::reconstitute(OrganizationSnapshot {
                id: id.value(),
                parent_id: None,
                name: "Testorg".into(),
            }))
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::default())
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
        }
    }

    fn service(identities: Vec<UserIdentity>, profiles: Arc<InMemoryProfiles>) -> UserService {
        let roles = Arc::new(StubRoles);
        UserService::new(
            Arc::new(StubIdentityRepo { identities }),
            profiles.clone(),
            profiles,
            roles.clone(),
            roles,
            Arc::new(StubOrgs),
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

        let page = svc
            .list(Pagination::default(), UserListFilter::default())
            .await
            .unwrap();

        assert_eq!(page.items[0].employee_id.as_deref(), Some("EMP-1"));
        assert_eq!(page.items[0].status, UserStatus::Absent);
        assert_eq!(page.items[0].driving_licenses, vec![DrivingLicense::CE]);
        assert_eq!(page.items[0].username.as_str(), "jdoe");
        assert!(page.items[0].organization.is_none());
        assert!(page.items[0].roles.is_empty());
    }

    #[tokio::test]
    async fn missing_profile_yields_defaults() {
        let id = Uuid::now_v7();
        let svc = service(
            vec![identity(id, "jdoe")],
            Arc::new(InMemoryProfiles::default()),
        );

        let page = svc
            .list(Pagination::default(), UserListFilter::default())
            .await
            .unwrap();

        assert_eq!(page.items[0].status, UserStatus::Available);
        assert!(page.items[0].driving_licenses.is_empty());
        assert!(page.items[0].employee_id.is_none());
    }

    #[tokio::test]
    async fn register_upserts_profile_and_sets_organization() {
        let profiles = Arc::new(InMemoryProfiles::default());
        let svc = service(vec![], profiles.clone());
        let org = Id::new_v7();
        let entity = UserCreate {
            username: Username::reconstitute("new".into()),
            first_name: "New".into(),
            last_name: "User".into(),
            email: Email::reconstitute("new@example.com".into()),
            password: SecretString::from("pw".to_string()),
            organization_id: org,
            role_ids: Vec::new(),
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
        let organization = view
            .organization
            .expect("organization present after register");
        assert_eq!(organization.id, org);
        assert_eq!(organization.name.as_str(), "Testorg");
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
