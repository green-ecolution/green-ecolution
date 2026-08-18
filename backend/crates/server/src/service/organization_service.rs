use std::sync::Arc;

use domain::{
    Id,
    events::DomainEvent,
    organization::{
        ContactPersonView, Organization, OrganizationDetailView, OrganizationDraft,
        OrganizationError, OrganizationName, OrganizationReader, OrganizationView,
        OrganizationWriter,
    },
    role::RoleReader,
    shared::address::Address,
    user::{UserProfileReader, UserRepository},
};
use uuid::Uuid;

use super::{ServiceError, event_bus::EventBus};

pub struct OrganizationService {
    org_reader: Arc<dyn OrganizationReader>,
    org_writer: Arc<dyn OrganizationWriter>,
    role_reader: Arc<dyn RoleReader>,
    profile_reader: Arc<dyn UserProfileReader>,
    user_repo: Arc<dyn UserRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl OrganizationService {
    pub fn new(
        org_reader: Arc<dyn OrganizationReader>,
        org_writer: Arc<dyn OrganizationWriter>,
        role_reader: Arc<dyn RoleReader>,
        profile_reader: Arc<dyn UserProfileReader>,
        user_repo: Arc<dyn UserRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            org_reader,
            org_writer,
            role_reader,
            profile_reader,
            user_repo,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self) -> Result<Vec<OrganizationView>, ServiceError> {
        let counts = self.org_reader.member_counts().await?;
        Ok(self
            .org_reader
            .all()
            .await?
            .iter()
            .map(|org| OrganizationView::new(org, counts.get(&org.id).copied().unwrap_or(0)))
            .collect())
    }

    /// Loads the whole count map for a single organization. The tree is small
    /// by design, so this stays cheaper than a dedicated per-id query.
    async fn view_of(&self, org: &Organization) -> Result<OrganizationView, ServiceError> {
        let counts = self.org_reader.member_counts().await?;
        Ok(OrganizationView::new(
            org,
            counts.get(&org.id).copied().unwrap_or(0),
        ))
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %id))]
    pub async fn detail(
        &self,
        id: Id<Organization>,
    ) -> Result<OrganizationDetailView, ServiceError> {
        let org = self.org_reader.by_id(id).await?;
        let organization = self.view_of(&org).await?;
        let contact_person = match org.contact_person() {
            None => None,
            Some(person) => self
                .user_repo
                .by_ids(&[person])
                .await?
                .into_iter()
                .next()
                .map(|identity| ContactPersonView {
                    id: identity.id,
                    first_name: identity.first_name,
                    last_name: identity.last_name,
                    email: identity.email,
                }),
        };
        Ok(OrganizationDetailView {
            organization,
            contact_person,
        })
    }

    /// Creates the organization and instantiates org-owned copies of every
    /// template in the same flow — a new org must never be observable without
    /// its default roles (hence no event handler for this).
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: OrganizationDraft) -> Result<OrganizationView, ServiceError> {
        let templates = self.role_reader.templates().await?;
        let org = self.org_writer.save_new(draft, templates).await?;
        self.event_bus
            .publish_all(vec![DomainEvent::OrganizationCreated {
                organization_id: org.id,
            }])
            .await;
        Ok(OrganizationView::new(&org, 0))
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %id))]
    pub async fn update(
        &self,
        id: Id<Organization>,
        name: OrganizationName,
        address: Option<Address>,
        contact_person: Option<Uuid>,
    ) -> Result<OrganizationView, ServiceError> {
        let mut org = self.org_reader.by_id(id).await?;
        // replace_details rejects the root before we touch the contact person,
        // so RootImmutable outranks a membership mismatch.
        let events = org.replace_details(name, address, contact_person)?;
        if let Some(person) = contact_person {
            self.ensure_member(id, person).await?;
        }
        self.org_writer.save(&org).await?;
        self.event_bus.publish_all(events).await;
        self.view_of(&org).await
    }

    /// `UserProfile` carries no organization, so membership is read from the
    /// dedicated mapping rather than from the profile itself.
    async fn ensure_member(&self, org: Id<Organization>, person: Uuid) -> Result<(), ServiceError> {
        let belongs = self
            .profile_reader
            .organizations_for(&[person])
            .await?
            .into_iter()
            .any(|(_, org_id)| org_id == org);
        if belongs {
            Ok(())
        } else {
            Err(ServiceError::ContactPersonNotAMember)
        }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %id))]
    pub async fn delete(&self, id: Id<Organization>) -> Result<(), ServiceError> {
        let org = self.org_reader.by_id(id).await?;
        if org.is_root() {
            return Err(OrganizationError::RootImmutable.into());
        }
        let has_children = self
            .org_reader
            .all()
            .await?
            .iter()
            .any(|o| o.parent_id() == Some(id));
        if has_children
            || !self
                .profile_reader
                .ids_in_organizations(&[id])
                .await?
                .is_empty()
        {
            return Err(ServiceError::OrganizationNotEmpty);
        }
        match self.org_writer.delete(id).await {
            // Resources still reference this org (trees, sensors, ...): the
            // pre-checks above only cover children and users.
            Err(domain::RepositoryError::ForeignKeyViolation(_)) => {
                return Err(ServiceError::OrganizationNotEmpty);
            }
            other => other?,
        }
        self.event_bus
            .publish_all(vec![DomainEvent::OrganizationDeleted {
                organization_id: id,
            }])
            .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::Utc;
    use uuid::Uuid;

    use domain::{
        RepositoryError,
        authorization::OrgHierarchy,
        organization::OrganizationSnapshot,
        role::{Role, RoleDraft, RoleSnapshot, RoleWriter},
        shared::{
            address::{City, PostalCode, Street},
            email::Email,
            pagination::{Page, Pagination},
        },
        user::{UserIdentity, UserIdentityCreate, UserProfile, Username},
    };

    struct InMemoryOrgs {
        rows: Mutex<Vec<Organization>>,
        saved_role_copies: Mutex<Vec<RoleDraft>>,
        member_counts: Mutex<HashMap<Id<Organization>, i64>>,
        fk_violation_on_delete: bool,
    }

    impl InMemoryOrgs {
        fn new(rows: Vec<Organization>) -> Self {
            Self {
                rows: Mutex::new(rows),
                saved_role_copies: Mutex::new(Vec::new()),
                member_counts: Mutex::new(HashMap::new()),
                fk_violation_on_delete: false,
            }
        }

        fn with_fk_violation_on_delete(rows: Vec<Organization>) -> Self {
            Self {
                fk_violation_on_delete: true,
                ..Self::new(rows)
            }
        }

        fn set_member_count(&self, org: Id<Organization>, count: i64) {
            self.member_counts.lock().unwrap().insert(org, count);
        }
    }

    #[async_trait::async_trait]
    impl OrganizationReader for InMemoryOrgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn by_id(&self, id: Id<Organization>) -> Result<Organization, RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .iter()
                .find(|o| o.id == id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs(Vec::new()))
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(self.member_counts.lock().unwrap().clone())
        }
    }

    #[async_trait::async_trait]
    impl OrganizationWriter for InMemoryOrgs {
        async fn save_new(
            &self,
            draft: OrganizationDraft,
            templates: Vec<Role>,
        ) -> Result<Organization, RepositoryError> {
            let org = Organization::reconstitute(OrganizationSnapshot {
                id: Uuid::now_v7(),
                parent_id: Some(draft.parent_id.value()),
                name: draft.name.as_str().to_string(),
                street: None,
                postal_code: None,
                city: None,
                contact_person_id: None,
            });
            self.rows.lock().unwrap().push(org.clone());
            let mut copies = self.saved_role_copies.lock().unwrap();
            copies.extend(templates.iter().map(|t| t.copy_for(org.id)));
            Ok(org)
        }
        async fn save(&self, org: &Organization) -> Result<(), RepositoryError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(existing) = rows.iter_mut().find(|o| o.id == org.id) {
                *existing = org.clone();
            }
            Ok(())
        }
        async fn delete(&self, id: Id<Organization>) -> Result<(), RepositoryError> {
            if self.fk_violation_on_delete {
                return Err(RepositoryError::ForeignKeyViolation(
                    "insert or update on table \"trees\" violates foreign key constraint \"trees_organization_id_fkey\""
                        .into(),
                ));
            }
            self.rows.lock().unwrap().retain(|o| o.id != id);
            Ok(())
        }
    }

    struct InMemoryRoles {
        templates: Vec<Role>,
        saved: Mutex<Vec<RoleDraft>>,
    }

    impl InMemoryRoles {
        fn new(templates: Vec<Role>) -> Self {
            Self {
                templates,
                saved: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl RoleReader for InMemoryRoles {
        async fn by_id(&self, _id: Id<Role>) -> Result<Role, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn by_organization(
            &self,
            _org: Id<Organization>,
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_organizations(
            &self,
            _orgs: &[Id<Organization>],
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
            Ok(self.templates.clone())
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
    impl RoleWriter for InMemoryRoles {
        async fn save_new(&self, draft: RoleDraft) -> Result<Role, RepositoryError> {
            self.saved.lock().unwrap().push(draft.clone());
            Ok(Role::reconstitute(RoleSnapshot {
                id: Uuid::now_v7(),
                organization_id: Some(draft.organization_id.value()),
                name: draft.name.as_str().to_string(),
                description: draft.description.as_ref().map(|d| d.as_str().to_string()),
                permissions: draft.permissions.iter().map(|p| p.to_string()).collect(),
            })
            .unwrap())
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

    #[derive(Default)]
    struct StubProfiles {
        ids_in_org: Vec<Uuid>,
        memberships: Vec<(Uuid, Id<Organization>)>,
    }

    impl StubProfiles {
        fn with_ids(ids: Vec<Uuid>) -> Self {
            Self {
                ids_in_org: ids,
                memberships: Vec::new(),
            }
        }

        fn with_membership(user: Uuid, org: Id<Organization>) -> Self {
            Self {
                ids_in_org: Vec::new(),
                memberships: vec![(user, org)],
            }
        }
    }

    #[async_trait::async_trait]
    impl UserProfileReader for StubProfiles {
        async fn by_ids(&self, _ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn ids_in_organizations(
            &self,
            _orgs: &[Id<Organization>],
        ) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(self.ids_in_org.clone())
        }
        async fn organizations_for(
            &self,
            ids: &[Uuid],
        ) -> Result<Vec<(Uuid, Id<Organization>)>, RepositoryError> {
            Ok(self
                .memberships
                .iter()
                .filter(|(user, _)| ids.contains(user))
                .cloned()
                .collect())
        }
    }

    struct StubUsers {
        identities: Vec<UserIdentity>,
    }

    impl StubUsers {
        fn with_person(id: Uuid, first_name: &str, last_name: &str) -> Self {
            Self {
                identities: vec![UserIdentity {
                    id,
                    created_at: Utc::now(),
                    username: Username::new(format!("{first_name}.{last_name}").to_lowercase())
                        .unwrap(),
                    first_name: first_name.to_string(),
                    last_name: last_name.to_string(),
                    email: Email::new(format!(
                        "{}.{}@flensburg.de",
                        first_name.to_lowercase(),
                        last_name.to_lowercase()
                    ))
                    .unwrap(),
                    email_verified: true,
                }],
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for StubUsers {
        async fn create(
            &self,
            _entity: UserIdentityCreate,
        ) -> Result<UserIdentity, RepositoryError> {
            unimplemented!("identity creation is not exercised by organization tests")
        }
        async fn all(
            &self,
            _pagination: Pagination,
        ) -> Result<Page<UserIdentity>, RepositoryError> {
            unimplemented!("identity listing is not exercised by organization tests")
        }
        async fn search(
            &self,
            _query: &str,
            _pagination: Pagination,
        ) -> Result<Page<UserIdentity>, RepositoryError> {
            unimplemented!("identity search is not exercised by organization tests")
        }
        async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserIdentity>, RepositoryError> {
            Ok(self
                .identities
                .iter()
                .filter(|identity| ids.contains(&identity.id))
                .cloned()
                .collect())
        }
    }

    #[derive(Default)]
    struct RecordingEventBus {
        events: Mutex<Vec<DomainEvent>>,
    }

    #[async_trait::async_trait]
    impl EventBus for RecordingEventBus {
        async fn publish(&self, event: DomainEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    fn org(id: Id<Organization>, parent: Option<Id<Organization>>) -> Organization {
        Organization::reconstitute(OrganizationSnapshot {
            id: id.value(),
            parent_id: parent.map(|p| p.value()),
            name: "Testorganisation".into(),
            street: None,
            postal_code: None,
            city: None,
            contact_person_id: None,
        })
    }

    fn org_name(name: &str) -> OrganizationName {
        OrganizationName::new(name).unwrap()
    }

    fn template_role(name: &str) -> Role {
        Role::reconstitute(RoleSnapshot {
            id: Uuid::now_v7(),
            organization_id: None,
            name: name.into(),
            description: None,
            permissions: Vec::new(),
        })
        .unwrap()
    }

    fn service(
        orgs: Arc<InMemoryOrgs>,
        roles: Arc<InMemoryRoles>,
        profiles: Arc<StubProfiles>,
        users: Arc<StubUsers>,
        bus: Arc<RecordingEventBus>,
    ) -> OrganizationService {
        OrganizationService::new(orgs.clone(), orgs, roles, profiles, users, bus)
    }

    fn no_users() -> Arc<StubUsers> {
        Arc::new(StubUsers {
            identities: Vec::new(),
        })
    }

    struct Fixture {
        service: OrganizationService,
        root_id: Id<Organization>,
        child_id: Id<Organization>,
        member_id: Uuid,
    }

    fn fixture_with_child() -> Fixture {
        let root_id = Id::new_v7();
        let child_id = Id::new_v7();
        let member_id = Uuid::now_v7();

        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root_id, None),
            org(child_id, Some(root_id)),
        ]));
        orgs.set_member_count(child_id, 1);

        let service = OrganizationService::new(
            orgs.clone(),
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::with_membership(member_id, child_id)),
            Arc::new(StubUsers::with_person(member_id, "Anke", "Kruse")),
            Arc::new(RecordingEventBus::default()),
        );

        Fixture {
            service,
            root_id,
            child_id,
            member_id,
        }
    }

    fn test_address() -> Address {
        Address::new(
            Street::new("Nordergraben 12").unwrap(),
            PostalCode::new("24937").unwrap(),
            City::new("Flensburg").unwrap(),
        )
    }

    #[tokio::test]
    async fn create_copies_every_template_into_the_new_org() {
        let root = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![org(root, None)]));
        let roles = Arc::new(InMemoryRoles::new(vec![
            template_role("Baumpflege"),
            template_role("Fuhrpark"),
        ]));
        let svc = service(
            orgs.clone(),
            roles,
            Arc::new(StubProfiles::default()),
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let view = svc
            .create(OrganizationDraft {
                name: org_name("TBZ Flensburg"),
                parent_id: root,
            })
            .await
            .unwrap();

        let saved = orgs.saved_role_copies.lock().unwrap();
        assert_eq!(saved.len(), 2);
        assert!(saved.iter().all(|d| d.organization_id == view.id));
    }

    #[tokio::test]
    async fn create_publishes_organization_created() {
        let root = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![org(root, None)]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            bus.clone(),
        );

        let view = svc
            .create(OrganizationDraft {
                name: org_name("TBZ Flensburg"),
                parent_id: root,
            })
            .await
            .unwrap();

        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::OrganizationCreated { organization_id } if organization_id == view.id
        ));
    }

    #[tokio::test]
    async fn update_stores_address_and_contact_person() {
        let fixture = fixture_with_child();
        let person = fixture.member_id;
        let view = fixture
            .service
            .update(
                fixture.child_id,
                OrganizationName::new("Stadtgärtnerei Nord").unwrap(),
                Some(test_address()),
                Some(person),
            )
            .await
            .unwrap();
        assert_eq!(view.name.as_str(), "Stadtgärtnerei Nord");
        assert_eq!(view.address, Some(test_address()));
        assert_eq!(view.contact_person_id, Some(person));
    }

    /// The outsider genuinely belongs to a *different* org, proving `ensure_member`
    /// compares organizations rather than merely finding no membership at all.
    #[tokio::test]
    async fn update_rejects_a_contact_person_from_another_organization() {
        let root_id = Id::new_v7();
        let child_id = Id::new_v7();
        let other_org_id = Id::new_v7();
        let outsider = Uuid::now_v7();

        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root_id, None),
            org(child_id, Some(root_id)),
            org(other_org_id, Some(root_id)),
        ]));
        let profiles = Arc::new(StubProfiles {
            ids_in_org: Vec::new(),
            memberships: vec![(outsider, other_org_id)],
        });
        let svc = OrganizationService::new(
            orgs.clone(),
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            profiles,
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let err = svc
            .update(
                child_id,
                OrganizationName::new("Stadtgärtnerei Nord").unwrap(),
                None,
                Some(outsider),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::ContactPersonNotAMember));
    }

    #[tokio::test]
    async fn update_rejects_the_root() {
        let fixture = fixture_with_child();
        let err = fixture
            .service
            .update(
                fixture.root_id,
                OrganizationName::new("Anders").unwrap(),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::Organization(OrganizationError::RootImmutable)
        ));
    }

    /// RootImmutable must win even when a contact person is also given —
    /// replace_details runs before ensure_member.
    #[tokio::test]
    async fn update_root_with_contact_person_reports_root_immutable() {
        let fixture = fixture_with_child();
        let outsider = Uuid::now_v7();
        let err = fixture
            .service
            .update(
                fixture.root_id,
                OrganizationName::new("Anders").unwrap(),
                None,
                Some(outsider),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::Organization(OrganizationError::RootImmutable)
        ));
    }

    #[tokio::test]
    async fn update_persists_and_publishes() {
        let root_id = Id::new_v7();
        let child_id = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root_id, None),
            org(child_id, Some(root_id)),
        ]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = OrganizationService::new(
            orgs.clone(),
            orgs.clone(),
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            bus.clone(),
        );

        svc.update(
            child_id,
            OrganizationName::new("Stadtgärtnerei Nord").unwrap(),
            Some(test_address()),
            None,
        )
        .await
        .unwrap();

        let stored = orgs
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.id == child_id)
            .cloned()
            .unwrap();
        assert_eq!(stored.name.as_str(), "Stadtgärtnerei Nord");
        assert_eq!(stored.address(), Some(&test_address()));

        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::OrganizationRenamed { organization_id } if organization_id == child_id
        ));
    }

    #[tokio::test]
    async fn list_carries_member_counts() {
        let fixture = fixture_with_child();
        let views = fixture.service.list().await.unwrap();
        let child = views.iter().find(|v| v.id == fixture.child_id).unwrap();
        assert_eq!(child.member_count, 1);
    }

    #[tokio::test]
    async fn detail_resolves_the_contact_person() {
        let fixture = fixture_with_child();
        fixture
            .service
            .update(
                fixture.child_id,
                OrganizationName::new("Stadtgärtnerei Nord").unwrap(),
                None,
                Some(fixture.member_id),
            )
            .await
            .unwrap();
        let detail = fixture.service.detail(fixture.child_id).await.unwrap();
        let person = detail.contact_person.unwrap();
        assert_eq!(person.id, fixture.member_id);
        assert_eq!(person.first_name, "Anke");
        assert_eq!(person.last_name, "Kruse");
    }

    /// The identity provider may not resolve a stored person (deleted there, or
    /// unreachable). The organization must still report the id it holds so a
    /// client can round-trip it instead of clearing the reference.
    #[tokio::test]
    async fn detail_keeps_the_stored_id_when_the_person_cannot_be_resolved() {
        let root_id = Id::new_v7();
        let child_id = Id::new_v7();
        let member_id = Uuid::now_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root_id, None),
            org(child_id, Some(root_id)),
        ]));
        let svc = OrganizationService::new(
            orgs.clone(),
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::with_membership(member_id, child_id)),
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        svc.update(
            child_id,
            OrganizationName::new("Stadtgärtnerei Nord").unwrap(),
            None,
            Some(member_id),
        )
        .await
        .unwrap();

        let detail = svc.detail(child_id).await.unwrap();
        assert!(detail.contact_person.is_none());
        assert_eq!(detail.organization.contact_person_id, Some(member_id));
    }

    #[tokio::test]
    async fn detail_without_contact_person_returns_none() {
        let fixture = fixture_with_child();
        let detail = fixture.service.detail(fixture.child_id).await.unwrap();
        assert!(detail.contact_person.is_none());
    }

    #[tokio::test]
    async fn delete_root_is_rejected() {
        let root = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![org(root, None)]));
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let result = svc.delete(root).await;

        assert!(matches!(
            result,
            Err(ServiceError::Organization(OrganizationError::RootImmutable))
        ));
    }

    #[tokio::test]
    async fn delete_with_children_is_rejected() {
        let root = Id::new_v7();
        let parent = Id::new_v7();
        let child = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root, None),
            org(parent, Some(root)),
            org(child, Some(parent)),
        ]));
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let result = svc.delete(parent).await;

        assert!(matches!(result, Err(ServiceError::OrganizationNotEmpty)));
    }

    #[tokio::test]
    async fn delete_with_users_is_rejected() {
        let root = Id::new_v7();
        let target = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root, None),
            org(target, Some(root)),
        ]));
        let profiles = Arc::new(StubProfiles::with_ids(vec![Uuid::now_v7()]));
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            profiles,
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let result = svc.delete(target).await;

        assert!(matches!(result, Err(ServiceError::OrganizationNotEmpty)));
    }

    #[tokio::test]
    async fn delete_with_resources_still_referencing_it_is_rejected() {
        let root = Id::new_v7();
        let target = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::with_fk_violation_on_delete(vec![
            org(root, None),
            org(target, Some(root)),
        ]));
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            Arc::new(RecordingEventBus::default()),
        );

        let result = svc.delete(target).await;

        assert!(matches!(result, Err(ServiceError::OrganizationNotEmpty)));
    }

    #[tokio::test]
    async fn delete_publishes_organization_deleted() {
        let root = Id::new_v7();
        let target = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root, None),
            org(target, Some(root)),
        ]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            orgs.clone(),
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            no_users(),
            bus.clone(),
        );

        svc.delete(target).await.unwrap();

        assert!(orgs.rows.lock().unwrap().iter().all(|o| o.id != target));
        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::OrganizationDeleted { organization_id } if organization_id == target
        ));
    }
}
