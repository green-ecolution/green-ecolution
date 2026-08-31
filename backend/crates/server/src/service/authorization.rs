use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    authorization::{
        AccessContext, Action, EffectivePermissions, Permission, Resource, Visibility,
    },
    organization::{Organization, OrganizationReader},
    role::{Role, RoleReader},
};

use super::{AuthError, ServiceError};

/// A pending change to a role definition, expressed as what the role would
/// still grant afterwards.
#[derive(Debug, Clone, Copy)]
pub enum RoleChange<'a> {
    PermissionsReplacedWith(&'a BTreeSet<Permission>),
    Deleted,
}

/// What the caller must keep in their own organization to stay able to repair
/// their own access: editing a role they already hold, and handing roles to
/// people. Losing either is irreversible on one's own account, because
/// `require_superset` refuses to grant back a permission the caller no longer
/// holds.
const SELF_ADMINISTRATION: [Permission; 2] = [
    Permission {
        resource: Resource::Role,
        action: Action::Update,
    },
    Permission {
        resource: Resource::User,
        action: Action::Update,
    },
];

/// The single place that answers "may user X do P in org O". Handlers call
/// this before invoking the domain services (Oskar lesson: scope evaluation
/// must live in exactly one spot).
pub struct AuthorizationService {
    org_reader: Arc<dyn OrganizationReader>,
    role_reader: Arc<dyn RoleReader>,
    enforced: bool,
}

impl AuthorizationService {
    pub fn new(
        org_reader: Arc<dyn OrganizationReader>,
        role_reader: Arc<dyn RoleReader>,
        enforced: bool,
    ) -> Self {
        Self {
            org_reader,
            role_reader,
            enforced,
        }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user.id = %user_id))]
    pub async fn context_for(&self, user_id: Uuid) -> Result<AccessContext, ServiceError> {
        if !self.enforced || user_id.is_nil() {
            return Ok(AccessContext::unrestricted());
        }
        let roles = self.role_reader.roles_for_user(user_id).await?;
        let grants = roles
            .into_iter()
            .filter_map(|r| {
                r.organization_id()
                    .map(|org| (org, r.permissions().clone()))
            })
            .collect();
        let hierarchy = self.org_reader.hierarchy().await?;
        Ok(AccessContext {
            permissions: EffectivePermissions::from_grants(grants),
            hierarchy,
        })
    }

    pub async fn require(
        &self,
        user_id: Uuid,
        permission: Permission,
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let ctx = self.context_for(user_id).await?;
        if ctx.allows_in(permission, org) {
            Ok(())
        } else {
            Err(AuthError::Forbidden.into())
        }
    }

    pub async fn require_superset(
        &self,
        user_id: Uuid,
        required: &BTreeSet<Permission>,
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        let ctx = self.context_for(user_id).await?;
        if ctx.superset_of(required, org) {
            Ok(())
        } else {
            Err(AuthError::Forbidden.into())
        }
    }

    pub fn enforced(&self) -> bool {
        self.enforced
    }

    /// Rejects a change to a role definition that would leave the caller
    /// without the rights to administer roles and members in their own
    /// organization.
    ///
    /// `ensure_not_self` (OP#3121) closes this dead end for role
    /// *assignments*; the definition is the second way in. Emptying or
    /// deleting the last role one holds strips the same rights, and the
    /// account is then only repairable at the database.
    ///
    /// Editing a role one holds stays allowed as long as the two rights in
    /// `SELF_ADMINISTRATION` survive — a blanket "hands off your own role"
    /// would block legitimate maintenance.
    #[tracing::instrument(level = "debug", skip_all, fields(user.id = %user_id, role.id = %role_id))]
    pub async fn ensure_administration_survives(
        &self,
        user_id: Uuid,
        own_org: Option<Id<Organization>>,
        role_id: Id<Role>,
        change: RoleChange<'_>,
    ) -> Result<(), ServiceError> {
        if !self.enforced || user_id.is_nil() {
            return Ok(());
        }
        let Some(own_org) = own_org else {
            return Ok(());
        };
        let roles = self.role_reader.roles_for_user(user_id).await?;
        if !roles.iter().any(|r| r.id == role_id) {
            return Ok(());
        }
        let grants = roles
            .into_iter()
            .filter_map(|r| {
                let permissions = if r.id == role_id {
                    match change {
                        RoleChange::Deleted => return None,
                        RoleChange::PermissionsReplacedWith(p) => p.clone(),
                    }
                } else {
                    r.permissions().clone()
                };
                r.organization_id().map(|org| (org, permissions))
            })
            .collect();
        let after = AccessContext {
            permissions: EffectivePermissions::from_grants(grants),
            hierarchy: self.org_reader.hierarchy().await?,
        };
        if SELF_ADMINISTRATION
            .iter()
            .all(|p| after.allows_in(*p, own_org))
        {
            Ok(())
        } else {
            Err(ServiceError::CannotRevokeOwnAdministration)
        }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user.id = %user_id))]
    pub async fn visible_orgs_for(
        &self,
        user_id: Uuid,
        permission: Permission,
    ) -> Result<Visibility, ServiceError> {
        Ok(self.context_for(user_id).await?.visible_orgs(permission))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        RepositoryError,
        authorization::{Action, OrgHierarchy, Resource},
        organization::Organization,
        role::{Role, RoleSnapshot},
    };

    struct StubOrgs {
        pairs: Vec<(Id<Organization>, Option<Id<Organization>>)>,
    }

    #[async_trait::async_trait]
    impl OrganizationReader for StubOrgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, _id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs(self.pairs.clone()))
        }
        async fn member_counts(
            &self,
        ) -> Result<std::collections::HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(std::collections::HashMap::new())
        }
    }

    struct StubRoles {
        by_user: Vec<(Uuid, Role)>,
    }

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
        async fn by_organizations(
            &self,
            _orgs: &[Id<Organization>],
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, RepositoryError> {
            Ok(self
                .by_user
                .iter()
                .filter(|(u, _)| *u == user_id)
                .map(|(_, r)| r.clone())
                .collect())
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

    fn role_in(org: Id<Organization>, perms: &[Permission]) -> Role {
        role_with_id(Id::new_v7(), org, perms)
    }

    fn role_with_id(id: Id<Role>, org: Id<Organization>, perms: &[Permission]) -> Role {
        Role::reconstitute(RoleSnapshot {
            id: id.value(),
            organization_id: Some(org.value()),
            name: "Testrolle".into(),
            description: None,
            permissions: perms.iter().map(|p| p.to_string()).collect(),
            template_key: None,
        })
        .unwrap()
    }

    fn tree_read() -> Permission {
        Permission::new(Resource::Tree, Action::Read)
    }

    fn role_update() -> Permission {
        Permission::new(Resource::Role, Action::Update)
    }

    fn user_update() -> Permission {
        Permission::new(Resource::User, Action::Update)
    }

    /// Two-level tree plus a service whose only user is `user`.
    fn guard_setup(
        org: Id<Organization>,
        root: Id<Organization>,
        by_user: Vec<(Uuid, Role)>,
        enforced: bool,
    ) -> AuthorizationService {
        AuthorizationService::new(
            Arc::new(StubOrgs {
                pairs: vec![(root, None), (org, Some(root))],
            }),
            Arc::new(StubRoles { by_user }),
            enforced,
        )
    }

    #[tokio::test]
    async fn require_allows_in_subtree_and_denies_upwards() {
        let (root, tbz, sub) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let user = Uuid::now_v7();
        let svc = AuthorizationService::new(
            Arc::new(StubOrgs {
                pairs: vec![(root, None), (tbz, Some(root)), (sub, Some(tbz))],
            }),
            Arc::new(StubRoles {
                by_user: vec![(user, role_in(tbz, &[tree_read()]))],
            }),
            true,
        );
        assert!(svc.require(user, tree_read(), sub).await.is_ok());
        assert!(matches!(
            svc.require(user, tree_read(), root).await,
            Err(ServiceError::Auth(AuthError::Forbidden))
        ));
    }

    #[tokio::test]
    async fn require_superset_denies_wider_permission_sets() {
        let (root, tbz) = (Id::new_v7(), Id::new_v7());
        let user = Uuid::now_v7();
        let svc = AuthorizationService::new(
            Arc::new(StubOrgs {
                pairs: vec![(root, None), (tbz, Some(root))],
            }),
            Arc::new(StubRoles {
                by_user: vec![(user, role_in(tbz, &[tree_read()]))],
            }),
            true,
        );
        let wider = BTreeSet::from([tree_read(), Permission::new(Resource::Tree, Action::Delete)]);
        assert!(
            svc.require_superset(user, &BTreeSet::from([tree_read()]), tbz)
                .await
                .is_ok()
        );
        assert!(svc.require_superset(user, &wider, tbz).await.is_err());
    }

    #[tokio::test]
    async fn demo_bypass_is_unrestricted() {
        let svc = AuthorizationService::new(
            Arc::new(StubOrgs { pairs: vec![] }),
            Arc::new(StubRoles { by_user: vec![] }),
            false,
        );
        assert!(
            svc.require(Uuid::nil(), tree_read(), Id::new_v7())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn visible_orgs_for_returns_subtree_or_unrestricted() {
        let (root, tbz) = (Id::new_v7(), Id::new_v7());
        let user = Uuid::now_v7();
        let svc = AuthorizationService::new(
            Arc::new(StubOrgs {
                pairs: vec![(root, None), (tbz, Some(root))],
            }),
            Arc::new(StubRoles {
                by_user: vec![(user, role_in(tbz, &[tree_read()]))],
            }),
            true,
        );
        match svc.visible_orgs_for(user, tree_read()).await.unwrap() {
            domain::authorization::Visibility::Only(orgs) => {
                assert!(orgs.contains(&tbz) && !orgs.contains(&root))
            }
            v => panic!("expected Only, got {v:?}"),
        }
        let demo = AuthorizationService::new(
            Arc::new(StubOrgs { pairs: vec![] }),
            Arc::new(StubRoles { by_user: vec![] }),
            false,
        );
        assert_eq!(
            demo.visible_orgs_for(Uuid::nil(), tree_read())
                .await
                .unwrap(),
            domain::authorization::Visibility::Unrestricted
        );
    }

    #[tokio::test]
    async fn emptying_your_own_last_admin_role_is_rejected() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(
                user,
                role_with_id(role_id, org, &[role_update(), user_update()]),
            )],
            true,
        );

        let emptied = BTreeSet::from([tree_read()]);
        let result = svc
            .ensure_administration_survives(
                user,
                Some(org),
                role_id,
                RoleChange::PermissionsReplacedWith(&emptied),
            )
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::CannotRevokeOwnAdministration)
        ));
    }

    #[tokio::test]
    async fn dropping_only_user_update_is_rejected_too() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(
                user,
                role_with_id(role_id, org, &[role_update(), user_update()]),
            )],
            true,
        );

        let without_user_update = BTreeSet::from([role_update()]);
        let result = svc
            .ensure_administration_survives(
                user,
                Some(org),
                role_id,
                RoleChange::PermissionsReplacedWith(&without_user_update),
            )
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::CannotRevokeOwnAdministration)
        ));
    }

    #[tokio::test]
    async fn deleting_your_own_last_admin_role_is_rejected() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(
                user,
                role_with_id(role_id, org, &[role_update(), user_update()]),
            )],
            true,
        );

        let result = svc
            .ensure_administration_survives(user, Some(org), role_id, RoleChange::Deleted)
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::CannotRevokeOwnAdministration)
        ));
    }

    #[tokio::test]
    async fn pruning_unrelated_permissions_stays_allowed() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(
                user,
                role_with_id(role_id, org, &[role_update(), user_update(), tree_read()]),
            )],
            true,
        );

        let kept = BTreeSet::from([role_update(), user_update()]);
        let result = svc
            .ensure_administration_survives(
                user,
                Some(org),
                role_id,
                RoleChange::PermissionsReplacedWith(&kept),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn a_second_role_carrying_the_rights_allows_the_change() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![
                (
                    user,
                    role_with_id(role_id, org, &[role_update(), user_update()]),
                ),
                (user, role_in(org, &[role_update(), user_update()])),
            ],
            true,
        );

        let result = svc
            .ensure_administration_survives(user, Some(org), role_id, RoleChange::Deleted)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn a_grant_from_the_parent_organization_keeps_the_change_allowed() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![
                (
                    user,
                    role_with_id(role_id, org, &[role_update(), user_update()]),
                ),
                (user, role_in(root, &[role_update(), user_update()])),
            ],
            true,
        );

        let result = svc
            .ensure_administration_survives(user, Some(org), role_id, RoleChange::Deleted)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn a_role_the_caller_does_not_hold_is_not_guarded() {
        let (root, org, foreign_role, user) =
            (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(user, role_in(org, &[role_update(), user_update()]))],
            true,
        );

        let result = svc
            .ensure_administration_survives(user, Some(org), foreign_role, RoleChange::Deleted)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn a_caller_without_an_organization_is_not_guarded() {
        let (root, org, role_id, user) = (Id::new_v7(), Id::new_v7(), Id::new_v7(), Uuid::now_v7());
        let svc = guard_setup(
            org,
            root,
            vec![(
                user,
                role_with_id(role_id, org, &[role_update(), user_update()]),
            )],
            true,
        );

        let result = svc
            .ensure_administration_survives(user, None, role_id, RoleChange::Deleted)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn demo_bypass_skips_the_guard() {
        let (root, org, role_id) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let svc = guard_setup(org, root, vec![], false);

        let result = svc
            .ensure_administration_survives(Uuid::nil(), Some(org), role_id, RoleChange::Deleted)
            .await;

        assert!(result.is_ok());
    }
}
