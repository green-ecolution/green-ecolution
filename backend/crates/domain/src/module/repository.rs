use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    module::{ActivationResolution, ModuleKey, OwnSwitch},
    organization::Organization,
};

#[async_trait]
pub trait ModuleActivationReader: Send + Sync {
    /// The organization's own switches. A module it never set is absent.
    async fn own(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, OwnSwitch>, RepositoryError>;
    /// One resolution per registered module; `NotFound` for an unknown organization.
    async fn resolve(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ActivationResolution>, RepositoryError>;
    /// Every organization in which `key` resolves to enabled. Empty for an
    /// unregistered key.
    async fn active_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError>;
}

#[async_trait]
pub trait ModuleActivationWriter: Send + Sync {
    /// `Some` sets the organization's own switch, stamped with `set_by`;
    /// `None` returns the module to inheritance.
    async fn apply(
        &self,
        org: Id<Organization>,
        changes: &BTreeMap<ModuleKey, Option<bool>>,
        set_by: Id<Organization>,
    ) -> Result<(), RepositoryError>;
}
