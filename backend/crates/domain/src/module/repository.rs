use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    module::{ActivationResolution, ModuleKey},
    organization::Organization,
};

#[async_trait]
pub trait ModuleActivationReader: Send + Sync {
    /// The organization's own switches. A module it never set is absent.
    async fn own(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, bool>, RepositoryError>;
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
    /// `Some` sets the organization's own switch, `None` returns the module to
    /// inheritance.
    async fn apply(
        &self,
        org: Id<Organization>,
        changes: &BTreeMap<ModuleKey, Option<bool>>,
    ) -> Result<(), RepositoryError>;
}
