use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    authorization::Visibility,
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

/// List filters. Organization membership and role assignments are resolved
/// against Postgres; the free-text query is resolved by the IdP, which owns
/// names and email addresses. `visibility` is not a client-supplied filter but
/// the caller's authorized scope: unlike the optional filters it can never be
/// widened, only narrowed further by them.
#[derive(Debug, Clone, Default)]
pub struct UserListFilter {
    pub organization_id: Option<Id<Organization>>,
    pub role_id: Option<Id<Role>>,
    pub query: Option<String>,
    pub visibility: Visibility,
}

/// How many IdP search hits are intersected against the locally filtered id
/// set at most. The two conditions live in different systems, so an exact
/// pagination across both is impossible. Reaching the cap truncates the
/// result, which the service logs rather than hiding.
pub const SEARCH_INTERSECTION_CAP: u64 = 500;

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
        // Org first: its INSERT satisfies the NOT NULL constraint, then upsert's
        // ON CONFLICT DO UPDATE only touches profile fields, leaving the org intact.
        self.profile_writer
            .set_organization(identity.id, entity.organization_id)
            .await?;
        self.profile_writer.update(&profile).await?;
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
        let org = self.org_reader.by_id(entity.organization_id).await?;
        let counts = self.org_reader.member_counts().await?;
        let organization = Some(OrganizationView::new(
            &org,
            counts.get(&org.id).copied().unwrap_or(0),
        ));
        Ok(merge(identity, Some(profile), organization, assigned))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(
        &self,
        pagination: Pagination,
        filter: UserListFilter,
    ) -> Result<Page<UserView>, ServiceError> {
        match (filter.query.as_deref(), self.candidate_ids(&filter).await?) {
            (None, None) => {
                if !self.enabled {
                    return Ok(demo_user_page(pagination, None));
                }
                let page = self.user_repo.all(pagination).await?;
                let items = self.attach_views(page.items).await?;
                Ok(Page {
                    items,
                    total: page.total,
                })
            }
            (None, Some(candidates)) => self.list_by_ids(pagination, candidates).await,
            (Some(query), None) => {
                if !self.enabled {
                    return Ok(demo_user_page(pagination, Some(query)));
                }
                let page = self.user_repo.search(query, pagination).await?;
                let items = self.attach_views(page.items).await?;
                Ok(Page {
                    items,
                    total: page.total,
                })
            }
            (Some(query), Some(candidates)) => {
                self.list_searched_and_filtered(pagination, query, candidates)
                    .await
            }
        }
    }

    async fn list_by_ids(
        &self,
        pagination: Pagination,
        candidates: HashSet<Uuid>,
    ) -> Result<Page<UserView>, ServiceError> {
        let mut ids: Vec<Uuid> = candidates.into_iter().collect();
        // Sorted so that `total` and every page slice describe the same
        // sequence — set iteration order would shuffle page boundaries.
        ids.sort_unstable();
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

    async fn list_searched_and_filtered(
        &self,
        pagination: Pagination,
        query: &str,
        allowed: HashSet<Uuid>,
    ) -> Result<Page<UserView>, ServiceError> {
        if allowed.is_empty() {
            return Ok(Page {
                items: Vec::new(),
                total: 0,
            });
        }

        let hits = if self.enabled {
            self.user_repo
                .search(
                    query,
                    Pagination::with_max_per_page(
                        1,
                        SEARCH_INTERSECTION_CAP,
                        SEARCH_INTERSECTION_CAP,
                    ),
                )
                .await?
        } else {
            let page = demo_user_page(Pagination::default(), Some(query));
            Page {
                items: page
                    .items
                    .iter()
                    .map(|view| demo_identity(view.id))
                    .collect(),
                total: page.total,
            }
        };

        if hits.total > SEARCH_INTERSECTION_CAP {
            tracing::warn!(
                cap = SEARCH_INTERSECTION_CAP,
                total = hits.total,
                "user search truncated before intersecting with the local filter"
            );
        }

        let matched: Vec<UserIdentity> = hits
            .items
            .into_iter()
            .filter(|identity| allowed.contains(&identity.id))
            .collect();
        let total = matched.len() as u64;
        let start = (pagination.offset() as usize).min(matched.len());
        let end = start
            .saturating_add(pagination.limit() as usize)
            .min(matched.len());
        let items = self.attach_views(matched[start..end].to_vec()).await?;
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
        self.profile_writer.update(&profile).await?;
        self.attach_views(vec![identity])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::NotFound.into())
    }

    /// The ids the result must be restricted to, or `None` when nothing
    /// restricts it and the IdP can supply the base population itself.
    ///
    /// A restricted visibility has to be resolved here rather than filtered
    /// afterwards: the IdP knows no organizations, so it cannot be asked for
    /// "the members of this subtree" — and filtering its already-paginated
    /// answer would both leak and misreport `total`.
    async fn candidate_ids(
        &self,
        filter: &UserListFilter,
    ) -> Result<Option<HashSet<Uuid>>, ServiceError> {
        let by_visibility = match &filter.visibility {
            Visibility::Unrestricted => None,
            Visibility::Only(orgs) => {
                let orgs: Vec<Id<Organization>> = orgs.iter().copied().collect();
                Some(self.profile_reader.ids_in_organizations(&orgs).await?)
            }
        };
        let by_org = match filter.organization_id {
            Some(org) => Some(self.profile_reader.ids_in_organizations(&[org]).await?),
            None => None,
        };
        let by_role = match filter.role_id {
            Some(role) => Some(self.role_reader.user_ids_with_role(role).await?),
            None => None,
        };
        let mut restricted: Option<HashSet<Uuid>> = None;
        for source in [by_visibility, by_org, by_role].into_iter().flatten() {
            restricted = Some(match restricted {
                None => source.into_iter().collect(),
                Some(keep) => source.into_iter().filter(|id| keep.contains(id)).collect(),
            });
        }
        Ok(restricted)
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
        let counts = self.org_reader.member_counts().await?;
        let mut orgs: HashMap<Id<Organization>, OrganizationView> = HashMap::new();
        for org_id in org_ids.values() {
            if !orgs.contains_key(org_id) {
                let org = self.org_reader.by_id(*org_id).await?;
                let count = counts.get(org_id).copied().unwrap_or(0);
                orgs.insert(*org_id, OrganizationView::new(&org, count));
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

/// In demo mode the single static user is the whole population, so a search
/// either matches it or yields nothing.
fn demo_user_page(pagination: Pagination, query: Option<&str>) -> Page<UserView> {
    let user = demo_user();
    let matches = match query {
        None => true,
        Some(q) => {
            let needle = q.to_lowercase();
            user.username.as_str().to_lowercase().contains(&needle)
                || user.first_name.to_lowercase().contains(&needle)
                || user.last_name.to_lowercase().contains(&needle)
                || user.email.as_str().to_lowercase().contains(&needle)
        }
    };
    if !matches {
        return Page {
            items: Vec::new(),
            total: 0,
        };
    }
    let items = if pagination.page() == 1 {
        vec![user]
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
    use std::{
        collections::{BTreeSet, HashMap},
        sync::Mutex,
    };

    struct StubIdentityRepo {
        identities: Vec<UserIdentity>,
        seen: Mutex<Vec<Pagination>>,
        total_override: Option<u64>,
    }

    impl StubIdentityRepo {
        fn new(identities: Vec<UserIdentity>) -> Self {
            Self {
                identities,
                seen: Mutex::new(Vec::new()),
                total_override: None,
            }
        }
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
        async fn search(
            &self,
            query: &str,
            pagination: Pagination,
        ) -> Result<Page<UserIdentity>, RepositoryError> {
            self.seen.lock().unwrap().push(pagination);
            let needle = query.to_lowercase();
            let items: Vec<UserIdentity> = self
                .identities
                .iter()
                .filter(|i| {
                    i.username.as_str().to_lowercase().contains(&needle)
                        || i.first_name.to_lowercase().contains(&needle)
                        || i.last_name.to_lowercase().contains(&needle)
                        || i.email.as_str().to_lowercase().contains(&needle)
                })
                .cloned()
                .collect();
            Ok(Page {
                total: self.total_override.unwrap_or(items.len() as u64),
                items,
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
        members: Mutex<HashMap<Id<Organization>, Vec<Uuid>>>,
    }

    impl InMemoryProfiles {
        fn add_member(&self, org: Id<Organization>, user: Uuid) {
            self.members
                .lock()
                .unwrap()
                .entry(org)
                .or_default()
                .push(user);
        }
    }

    #[async_trait::async_trait]
    impl UserProfileReader for InMemoryProfiles {
        async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError> {
            let rows = self.rows.lock().unwrap();
            Ok(ids.iter().filter_map(|id| rows.get(id).cloned()).collect())
        }
        async fn ids_in_organizations(
            &self,
            orgs: &[Id<Organization>],
        ) -> Result<Vec<Uuid>, RepositoryError> {
            let members = self.members.lock().unwrap();
            Ok(orgs
                .iter()
                .filter_map(|org| members.get(org))
                .flatten()
                .copied()
                .collect())
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
        async fn update(&self, profile: &UserProfile) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .insert(profile.id, profile.clone());
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
                street: None,
                postal_code: None,
                city: None,
                contact_person_id: None,
            }))
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::default())
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(HashMap::new())
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

    fn service_with_repo(
        repo: Arc<StubIdentityRepo>,
        profiles: Arc<InMemoryProfiles>,
    ) -> UserService {
        let roles = Arc::new(StubRoles);
        UserService::new(
            repo,
            profiles.clone(),
            profiles,
            roles.clone(),
            roles,
            Arc::new(StubOrgs),
            true,
        )
    }

    fn service(identities: Vec<UserIdentity>, profiles: Arc<InMemoryProfiles>) -> UserService {
        service_with_repo(Arc::new(StubIdentityRepo::new(identities)), profiles)
    }

    #[tokio::test]
    async fn list_merges_profile_into_view() {
        let id = Uuid::now_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles
            .update(&UserProfile {
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

    #[tokio::test]
    async fn search_matches_last_name_and_ignores_the_rest() {
        let anna = Uuid::now_v7();
        let bo = Uuid::now_v7();
        let svc = service(
            vec![identity(anna, "aahlmann"), identity(bo, "bboysen")],
            Arc::new(InMemoryProfiles::default()),
        );

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    query: Some("bboysen".into()),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].username.as_str(), "bboysen");
    }

    #[tokio::test]
    async fn search_combined_with_organization_filter_yields_the_intersection() {
        let inside = Uuid::now_v7();
        let outside = Uuid::now_v7();
        let org = Id::new_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(org, inside);
        let repo = Arc::new(StubIdentityRepo::new(vec![
            identity(inside, "jdoe"),
            identity(outside, "jdoe2"),
        ]));
        let svc = service_with_repo(repo.clone(), profiles);

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    organization_id: Some(org),
                    query: Some("jdoe".into()),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        // Both identities match the query, only one is a member of the organization.
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, inside);

        let seen = repo.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].per_page(), SEARCH_INTERSECTION_CAP);
    }

    #[tokio::test]
    async fn search_intersection_is_correct_even_when_the_idp_total_is_truncated() {
        let inside = Uuid::now_v7();
        let org = Id::new_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(org, inside);
        let mut repo = StubIdentityRepo::new(vec![identity(inside, "jdoe")]);
        repo.total_override = Some(SEARCH_INTERSECTION_CAP + 1);
        let svc = service_with_repo(Arc::new(repo), profiles);

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    organization_id: Some(org),
                    query: Some("jdoe".into()),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        // hits.total (truncated, > SEARCH_INTERSECTION_CAP) must not leak into the
        // reported page total; the service reports the intersected count instead.
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, inside);
    }

    #[tokio::test]
    async fn restricted_visibility_replaces_the_idp_as_the_base_population() {
        let inside = Uuid::now_v7();
        let outside = Uuid::now_v7();
        let visible_org = Id::new_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(visible_org, inside);
        profiles.add_member(Id::new_v7(), outside);
        let svc = service(
            vec![identity(inside, "inside"), identity(outside, "outside")],
            profiles,
        );

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    visibility: Visibility::Only(BTreeSet::from([visible_org])),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        // The IdP knows both identities; only the locally visible one may show up.
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, inside);
    }

    #[tokio::test]
    async fn a_filter_on_an_invisible_organization_yields_an_empty_page() {
        let inside = Uuid::now_v7();
        let elsewhere = Uuid::now_v7();
        let visible_org = Id::new_v7();
        let invisible_org = Id::new_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(visible_org, inside);
        profiles.add_member(invisible_org, elsewhere);
        let svc = service(
            vec![identity(inside, "inside"), identity(elsewhere, "elsewhere")],
            profiles,
        );

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    organization_id: Some(invisible_org),
                    visibility: Visibility::Only(BTreeSet::from([visible_org])),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }

    #[tokio::test]
    async fn a_visibility_granting_nothing_yields_an_empty_page() {
        let user = Uuid::now_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(Id::new_v7(), user);
        let svc = service(vec![identity(user, "somebody")], profiles);

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    visibility: Visibility::Only(BTreeSet::new()),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        // An empty scope must not read as "no restriction".
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }

    #[tokio::test]
    async fn search_is_intersected_with_a_restricted_visibility() {
        let inside = Uuid::now_v7();
        let outside = Uuid::now_v7();
        let visible_org = Id::new_v7();
        let profiles = Arc::new(InMemoryProfiles::default());
        profiles.add_member(visible_org, inside);
        profiles.add_member(Id::new_v7(), outside);
        let svc = service(
            vec![identity(inside, "jdoe"), identity(outside, "jdoe2")],
            profiles,
        );

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    query: Some("jdoe".into()),
                    visibility: Visibility::Only(BTreeSet::from([visible_org])),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, inside);
    }

    #[tokio::test]
    async fn search_without_hits_yields_an_empty_page() {
        let svc = service(
            vec![identity(Uuid::now_v7(), "jdoe")],
            Arc::new(InMemoryProfiles::default()),
        );

        let page = svc
            .list(
                Pagination::default(),
                UserListFilter {
                    query: Some("zzz".into()),
                    ..UserListFilter::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }
}
