use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    authorization::{AccessContext, EffectivePermissions, Permission, Visibility},
    organization::{Organization, OrganizationReader},
    role::RoleReader,
};

use super::{AuthError, ServiceError};

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
        Role::reconstitute(RoleSnapshot {
            id: Uuid::now_v7(),
            organization_id: Some(org.value()),
            name: "Testrolle".into(),
            description: None,
            permissions: perms.iter().map(|p| p.to_string()).collect(),
        })
        .unwrap()
    }

    fn tree_read() -> Permission {
        Permission::new(Resource::Tree, Action::Read)
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
}
