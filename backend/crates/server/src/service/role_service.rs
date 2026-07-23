use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    authorization::Permission,
    events::DomainEvent,
    organization::Organization,
    role::{
        Role, RoleDescription, RoleDraft, RoleError, RoleName, RoleReader, RoleView, RoleWriter,
    },
    user::UserProfileWriter,
};

use super::{ServiceError, event_bus::EventBus};

pub struct RoleService {
    role_reader: Arc<dyn RoleReader>,
    role_writer: Arc<dyn RoleWriter>,
    profile_writer: Arc<dyn UserProfileWriter>,
    event_bus: Arc<dyn EventBus>,
}

impl RoleService {
    pub fn new(
        role_reader: Arc<dyn RoleReader>,
        role_writer: Arc<dyn RoleWriter>,
        profile_writer: Arc<dyn UserProfileWriter>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self {
            role_reader,
            role_writer,
            profile_writer,
            event_bus,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn templates(&self) -> Result<Vec<RoleView>, ServiceError> {
        Ok(self
            .role_reader
            .templates()
            .await?
            .iter()
            .map(RoleView::from)
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(organization.id = %org))]
    pub async fn by_organization(
        &self,
        org: Id<Organization>,
    ) -> Result<Vec<RoleView>, ServiceError> {
        Ok(self
            .role_reader
            .by_organization(org)
            .await?
            .iter()
            .map(RoleView::from)
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %id))]
    pub async fn by_id(&self, id: Id<Role>) -> Result<RoleView, ServiceError> {
        Ok((&self.role_reader.by_id(id).await?).into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: RoleDraft) -> Result<RoleView, ServiceError> {
        let org = draft.organization_id;
        let role = self.role_writer.save_new(draft).await?;
        self.publish_created(&role, org).await;
        Ok((&role).into())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %source))]
    pub async fn copy(
        &self,
        source: Id<Role>,
        target_org: Id<Organization>,
    ) -> Result<RoleView, ServiceError> {
        let source_role = self.role_reader.by_id(source).await?;
        let role = self
            .role_writer
            .save_new(source_role.copy_for(target_org))
            .await?;
        self.publish_created(&role, target_org).await;
        Ok((&role).into())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %id))]
    pub async fn update(
        &self,
        id: Id<Role>,
        name: RoleName,
        description: Option<RoleDescription>,
        permissions: BTreeSet<Permission>,
    ) -> Result<RoleView, ServiceError> {
        let mut role = self.role_reader.by_id(id).await?;
        let mut events = role.rename(name)?;
        role.set_description(description)?;
        events.extend(role.replace_permissions(permissions)?);
        self.role_writer.save(&role).await?;
        self.event_bus.publish_all(events).await;
        Ok((&role).into())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %id))]
    pub async fn delete(&self, id: Id<Role>) -> Result<(), ServiceError> {
        let role = self.role_reader.by_id(id).await?;
        if role.is_template() {
            return Err(RoleError::TemplateImmutable.into());
        }
        self.role_writer.delete(id).await?;
        self.event_bus
            .publish_all(vec![DomainEvent::RoleDeleted { role_id: id }])
            .await;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %role_id, user.id = %user_id))]
    pub async fn assign(&self, user_id: Uuid, role_id: Id<Role>) -> Result<(), ServiceError> {
        let role = self.role_reader.by_id(role_id).await?;
        if role.is_template() {
            return Err(RoleError::CannotAssignTemplate.into());
        }
        self.profile_writer.ensure_exists(user_id).await?;
        self.role_writer.assign_to_user(user_id, role_id).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(role.id = %role_id, user.id = %user_id))]
    pub async fn revoke(&self, user_id: Uuid, role_id: Id<Role>) -> Result<(), ServiceError> {
        self.role_writer.revoke_from_user(user_id, role_id).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user.id = %user_id))]
    pub async fn roles_of_user(&self, user_id: Uuid) -> Result<Vec<RoleView>, ServiceError> {
        Ok(self
            .role_reader
            .roles_for_user(user_id)
            .await?
            .iter()
            .map(RoleView::from)
            .collect())
    }

    async fn publish_created(&self, role: &Role, org: Id<Organization>) {
        self.event_bus
            .publish_all(vec![DomainEvent::RoleCreated {
                role_id: role.id,
                organization_id: org,
            }])
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use domain::{
        RepositoryError,
        authorization::{Action, Resource},
        role::RoleSnapshot,
        user::UserProfile,
    };

    struct InMemoryRoles {
        rows: Mutex<Vec<Role>>,
        saved_new: Mutex<Vec<RoleDraft>>,
        saved: Mutex<Vec<Role>>,
        deleted: Mutex<Vec<Id<Role>>>,
        assigned: Mutex<Vec<(Uuid, Id<Role>)>>,
    }

    impl InMemoryRoles {
        fn new(rows: Vec<Role>) -> Self {
            Self {
                rows: Mutex::new(rows),
                saved_new: Mutex::new(Vec::new()),
                saved: Mutex::new(Vec::new()),
                deleted: Mutex::new(Vec::new()),
                assigned: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl RoleReader for InMemoryRoles {
        async fn by_id(&self, id: Id<Role>) -> Result<Role, RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }
        async fn by_organization(
            &self,
            org: Id<Organization>,
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.organization_id() == Some(org))
                .cloned()
                .collect())
        }
        async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.is_template())
                .cloned()
                .collect())
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
            self.saved_new.lock().unwrap().push(draft.clone());
            let role = Role::reconstitute(RoleSnapshot {
                id: Uuid::now_v7(),
                organization_id: Some(draft.organization_id.value()),
                name: draft.name.as_str().to_string(),
                description: draft.description.as_ref().map(|d| d.as_str().to_string()),
                permissions: draft.permissions.iter().map(|p| p.to_string()).collect(),
            })
            .unwrap();
            self.rows.lock().unwrap().push(role.clone());
            Ok(role)
        }
        async fn save(&self, role: &Role) -> Result<(), RepositoryError> {
            self.saved.lock().unwrap().push(role.clone());
            let mut rows = self.rows.lock().unwrap();
            if let Some(existing) = rows.iter_mut().find(|r| r.id == role.id) {
                *existing = role.clone();
            }
            Ok(())
        }
        async fn delete(&self, id: Id<Role>) -> Result<(), RepositoryError> {
            self.deleted.lock().unwrap().push(id);
            self.rows.lock().unwrap().retain(|r| r.id != id);
            Ok(())
        }
        async fn assign_to_user(
            &self,
            user_id: Uuid,
            role_id: Id<Role>,
        ) -> Result<(), RepositoryError> {
            self.assigned.lock().unwrap().push((user_id, role_id));
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
    struct InMemoryProfiles {
        ensured: Mutex<Vec<Uuid>>,
    }

    #[async_trait::async_trait]
    impl domain::user::UserProfileWriter for InMemoryProfiles {
        async fn upsert(&self, _profile: &UserProfile) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn ensure_exists(&self, id: Uuid) -> Result<(), RepositoryError> {
            self.ensured.lock().unwrap().push(id);
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
    struct RecordingEventBus {
        events: Mutex<Vec<DomainEvent>>,
    }

    #[async_trait::async_trait]
    impl EventBus for RecordingEventBus {
        async fn publish(&self, event: DomainEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    fn perms() -> BTreeSet<Permission> {
        BTreeSet::from([Permission::new(Resource::Tree, Action::Read)])
    }

    fn role(id: Id<Role>, org: Option<Id<Organization>>) -> Role {
        Role::reconstitute(RoleSnapshot {
            id: id.value(),
            organization_id: org.map(|o| o.value()),
            name: "Baumpflege".into(),
            description: None,
            permissions: perms().iter().map(|p| p.to_string()).collect(),
        })
        .unwrap()
    }

    fn service(
        roles: Arc<InMemoryRoles>,
        profiles: Arc<InMemoryProfiles>,
        bus: Arc<RecordingEventBus>,
    ) -> RoleService {
        RoleService::new(roles.clone(), roles, profiles, bus)
    }

    #[tokio::test]
    async fn create_publishes_role_created() {
        let roles = Arc::new(InMemoryRoles::new(Vec::new()));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            roles.clone(),
            Arc::new(InMemoryProfiles::default()),
            bus.clone(),
        );
        let org = Id::new_v7();

        let view = svc
            .create(RoleDraft {
                organization_id: org,
                name: RoleName::new("Baumpflege").unwrap(),
                description: None,
                permissions: perms(),
            })
            .await
            .unwrap();

        assert_eq!(roles.saved_new.lock().unwrap().len(), 1);
        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::RoleCreated { role_id, organization_id }
                if role_id == view.id && organization_id == org
        ));
    }

    #[tokio::test]
    async fn copy_creates_org_bound_copy_of_a_template() {
        let template_id = Id::new_v7();
        let template = role(template_id, None);
        let roles = Arc::new(InMemoryRoles::new(vec![template]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            roles.clone(),
            Arc::new(InMemoryProfiles::default()),
            bus.clone(),
        );
        let target_org = Id::new_v7();

        let view = svc.copy(template_id, target_org).await.unwrap();

        let saved = roles.saved_new.lock().unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].organization_id, target_org);
        assert_eq!(saved[0].permissions, perms());
        assert_eq!(view.organization_id, Some(target_org));

        let events = bus.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::RoleCreated { organization_id, .. } if organization_id == target_org
        ));
    }

    #[tokio::test]
    async fn update_applies_rename_description_and_permissions() {
        let id = Id::new_v7();
        let org = Id::new_v7();
        let roles = Arc::new(InMemoryRoles::new(vec![role(id, Some(org))]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            roles.clone(),
            Arc::new(InMemoryProfiles::default()),
            bus.clone(),
        );

        let new_name = RoleName::new("Pflege Nord").unwrap();
        let new_description = Some(RoleDescription::new("Neue Beschreibung").unwrap());
        let new_permissions = BTreeSet::from([Permission::new(Resource::Vehicle, Action::Update)]);

        let view = svc
            .update(
                id,
                new_name.clone(),
                new_description.clone(),
                new_permissions.clone(),
            )
            .await
            .unwrap();

        assert_eq!(view.name, new_name);
        assert_eq!(view.description, new_description);
        assert_eq!(
            view.permissions.iter().copied().collect::<BTreeSet<_>>(),
            new_permissions
        );

        assert_eq!(roles.saved.lock().unwrap().len(), 1);
        let events = bus.events.lock().unwrap();
        assert!(matches!(events[0], DomainEvent::RoleRenamed { role_id } if role_id == id));
        assert!(matches!(
            events[1],
            DomainEvent::RolePermissionsChanged { role_id } if role_id == id
        ));
    }

    #[tokio::test]
    async fn update_on_template_is_rejected() {
        let id = Id::new_v7();
        let roles = Arc::new(InMemoryRoles::new(vec![role(id, None)]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            roles.clone(),
            Arc::new(InMemoryProfiles::default()),
            bus.clone(),
        );

        let result = svc
            .update(id, RoleName::new("Anders").unwrap(), None, BTreeSet::new())
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::Role(RoleError::TemplateImmutable))
        ));
        assert!(roles.saved.lock().unwrap().is_empty());
        assert!(bus.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_on_template_is_rejected() {
        let id = Id::new_v7();
        let roles = Arc::new(InMemoryRoles::new(vec![role(id, None)]));
        let bus = Arc::new(RecordingEventBus::default());
        let svc = service(
            roles.clone(),
            Arc::new(InMemoryProfiles::default()),
            bus.clone(),
        );

        let result = svc.delete(id).await;

        assert!(matches!(
            result,
            Err(ServiceError::Role(RoleError::TemplateImmutable))
        ));
        assert!(roles.deleted.lock().unwrap().is_empty());
        assert!(bus.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn assign_rejects_templates_and_ensures_profile_row() {
        let template_id = Id::new_v7();
        let org_role_id = Id::new_v7();
        let org = Id::new_v7();
        let roles = Arc::new(InMemoryRoles::new(vec![
            role(template_id, None),
            role(org_role_id, Some(org)),
        ]));
        let profiles = Arc::new(InMemoryProfiles::default());
        let svc = service(
            roles.clone(),
            profiles.clone(),
            Arc::new(RecordingEventBus::default()),
        );
        let user_id = Uuid::now_v7();

        let template_result = svc.assign(user_id, template_id).await;
        assert!(matches!(
            template_result,
            Err(ServiceError::Role(RoleError::CannotAssignTemplate))
        ));
        assert!(profiles.ensured.lock().unwrap().is_empty());
        assert!(roles.assigned.lock().unwrap().is_empty());

        svc.assign(user_id, org_role_id).await.unwrap();
        assert_eq!(profiles.ensured.lock().unwrap().as_slice(), [user_id]);
        assert_eq!(
            roles.assigned.lock().unwrap().as_slice(),
            [(user_id, org_role_id)]
        );
    }
}
