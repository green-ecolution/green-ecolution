use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    module::{ModuleKey, ModuleState},
    organization::Organization,
};

#[async_trait]
pub trait ModuleActivationReader: Send + Sync {
    /// One state per registered module, taken from the organization's tenant;
    /// `NotFound` for an unknown organization.
    async fn resolve(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ModuleState>, RepositoryError>;
    /// Every organization in which `key` is released and switched on. Empty
    /// for an unregistered key.
    async fn active_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError>;
    /// Every organization in which `key` is released, switched on or not.
    async fn released_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError>;
    /// Every tenant with the state of each registered module.
    async fn tenants(
        &self,
    ) -> Result<BTreeMap<Id<Organization>, BTreeMap<ModuleKey, ModuleState>>, RepositoryError>;
}

#[async_trait]
pub trait ModuleActivationWriter: Send + Sync {
    /// Switches modules on or off for `holder`, a tenant or the root. The
    /// caller has checked that each module is released there.
    async fn switch(
        &self,
        holder: Id<Organization>,
        changes: &BTreeMap<ModuleKey, bool>,
    ) -> Result<(), RepositoryError>;
    /// Releases (`true`) or locks (`false`) modules for a tenant. Locking
    /// keeps the tenant's switch, see `released_state`.
    async fn release(
        &self,
        tenant: Id<Organization>,
        changes: &BTreeMap<ModuleKey, bool>,
    ) -> Result<(), RepositoryError>;
}
