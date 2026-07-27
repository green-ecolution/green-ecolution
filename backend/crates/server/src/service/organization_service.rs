use std::sync::Arc;

use domain::{
    Id,
    events::DomainEvent,
    organization::{
        Organization, OrganizationDraft, OrganizationError, OrganizationName, OrganizationReader,
        OrganizationView, OrganizationWriter,
    },
    role::RoleReader,
    user::UserProfileReader,
};

use super::{ServiceError, event_bus::EventBus};

pub struct OrganizationService {
    org_reader: Arc<dyn OrganizationReader>,
    org_writer: Arc<dyn OrganizationWriter>,
    role_reader: Arc<dyn RoleReader>,
    profile_reader: Arc<dyn UserProfileReader>,
    event_bus: Arc<dyn EventBus>,
}

impl OrganizationService {
    pub fn new(
        org_reader: Arc<dyn OrganizationReader>,
        org_writer: Arc<dyn OrganizationWriter>,
        role_reader: Arc<dyn RoleReader>,
        profile_reader: Arc<dyn UserProfileReader>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            org_reader,
            org_writer,
            role_reader,
            profile_reader,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self) -> Result<Vec<OrganizationView>, ServiceError> {
        Ok(self
            .org_reader
            .all()
            .await?
            .iter()
            .map(OrganizationView::from)
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %id))]
    pub async fn by_id(&self, id: Id<Organization>) -> Result<OrganizationView, ServiceError> {
        Ok((&self.org_reader.by_id(id).await?).into())
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
        Ok((&org).into())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %id))]
    pub async fn rename(
        &self,
        id: Id<Organization>,
        name: OrganizationName,
    ) -> Result<OrganizationView, ServiceError> {
        let mut org = self.org_reader.by_id(id).await?;
        let events = org.rename(name)?;
        self.org_writer.save(&org).await?;
        self.event_bus.publish_all(events).await;
        Ok((&org).into())
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
                .ids_in_organization(id)
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
    use std::sync::Mutex;

    use uuid::Uuid;

    use domain::{
        RepositoryError,
        authorization::OrgHierarchy,
        organization::OrganizationSnapshot,
        role::{Role, RoleDraft, RoleSnapshot, RoleWriter},
        user::UserProfile,
    };

    struct InMemoryOrgs {
        rows: Mutex<Vec<Organization>>,
        saved_role_copies: Mutex<Vec<RoleDraft>>,
        fk_violation_on_delete: bool,
    }

    impl InMemoryOrgs {
        fn new(rows: Vec<Organization>) -> Self {
            Self {
                rows: Mutex::new(rows),
                saved_role_copies: Mutex::new(Vec::new()),
                fk_violation_on_delete: false,
            }
        }

        fn with_fk_violation_on_delete(rows: Vec<Organization>) -> Self {
            Self {
                fk_violation_on_delete: true,
                ..Self::new(rows)
            }
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
    }

    impl StubProfiles {
        fn with_ids(ids: Vec<Uuid>) -> Self {
            Self { ids_in_org: ids }
        }
    }

    #[async_trait::async_trait]
    impl UserProfileReader for StubProfiles {
        async fn by_ids(&self, _ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn ids_in_organization(
            &self,
            _org: Id<Organization>,
        ) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(self.ids_in_org.clone())
        }
        async fn organizations_for(
            &self,
            _ids: &[Uuid],
        ) -> Result<Vec<(Uuid, Id<Organization>)>, RepositoryError> {
            Ok(Vec::new())
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
        bus: Arc<RecordingEventBus>,
    ) -> OrganizationService {
        OrganizationService::new(orgs.clone(), orgs, roles, profiles, bus)
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
    async fn rename_persists_and_publishes() {
        let root = Id::new_v7();
        let id = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![
            org(root, None),
            org(id, Some(root)),
        ]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            orgs.clone(),
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
            bus.clone(),
        );

        let view = svc.rename(id, org_name("Neuer Name")).await.unwrap();

        assert_eq!(view.name.as_str(), "Neuer Name");
        assert_eq!(
            orgs.rows
                .lock()
                .unwrap()
                .iter()
                .find(|o| o.id == id)
                .unwrap()
                .name
                .as_str(),
            "Neuer Name"
        );
        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::OrganizationRenamed { organization_id } if organization_id == id
        ));
    }

    #[tokio::test]
    async fn delete_root_is_rejected() {
        let root = Id::new_v7();
        let orgs = Arc::new(InMemoryOrgs::new(vec![org(root, None)]));
        let svc = service(
            orgs,
            Arc::new(InMemoryRoles::new(Vec::new())),
            Arc::new(StubProfiles::default()),
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
