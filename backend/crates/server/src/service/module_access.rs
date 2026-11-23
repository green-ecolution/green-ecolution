use std::sync::Arc;

use domain::{
    Id,
    authorization::{Caller, Permission, Visibility},
    module::{ModuleActivationReader, ModuleKey},
    organization::Organization,
};

use crate::service::{
    ServiceError, authorization::AuthorizationService, targets::ensure_module_active,
};

/// The one place a module asks "may this user do this here". It adds the
/// module's activation to the permission check so no module can forget it.
#[derive(Clone)]
pub struct ModuleAccess {
    key: ModuleKey,
    authorization: Arc<AuthorizationService>,
    activation: Arc<dyn ModuleActivationReader>,
}

impl ModuleAccess {
    pub fn new(
        key: ModuleKey,
        authorization: Arc<AuthorizationService>,
        activation: Arc<dyn ModuleActivationReader>,
    ) -> Self {
        Self {
            key,
            authorization,
            activation,
        }
    }

    pub fn key(&self) -> &ModuleKey {
        &self.key
    }

    pub async fn ensure_active(&self, org: Id<Organization>) -> Result<(), ServiceError> {
        ensure_module_active(self.activation.as_ref(), &self.key, org).await
    }

    /// Route gate: the module must be released in at least one organization the
    /// caller reaches. Whether it is switched on is checked per organization.
    pub async fn ensure_released(&self, caller: Caller) -> Result<(), ServiceError> {
        let reachable = self
            .authorization
            .context_for(caller)
            .await?
            .reachable_orgs();
        let released = self.activation.released_orgs(&self.key).await?;
        let reached = match reachable {
            Visibility::Unrestricted => !released.is_empty(),
            Visibility::Only(ids) => !ids.is_disjoint(&released),
        };
        if reached {
            Ok(())
        } else {
            Err(ServiceError::ModuleDisabled {
                module: self.key.to_string(),
            })
        }
    }

    pub async fn require_write(
        &self,
        caller: Caller,
        permission: Permission,
        org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        self.authorization.require(caller, permission, org).await?;
        ensure_module_active(self.activation.as_ref(), &self.key, org).await
    }

    /// Never `Unrestricted`: even the demo bypass only sees organizations in
    /// which the module is active.
    pub async fn visible_orgs(
        &self,
        caller: Caller,
        permission: Permission,
    ) -> Result<Visibility, ServiceError> {
        let visible = self
            .authorization
            .visible_orgs_for(caller, permission)
            .await?;
        let active = self.activation.active_orgs(&self.key).await?;
        Ok(Visibility::Only(match visible {
            Visibility::Unrestricted => active,
            Visibility::Only(ids) => ids.intersection(&active).copied().collect(),
        }))
    }
}
