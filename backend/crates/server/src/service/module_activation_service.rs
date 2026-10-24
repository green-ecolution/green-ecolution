use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use domain::{
    Id,
    authorization::Visibility,
    module::{ActivationResolution, ModuleActivationReader, ModuleActivationWriter, ModuleKey},
    organization::{Organization, OrganizationReader},
};

use crate::service::ServiceError;

pub struct ModuleActivationService {
    reader: Arc<dyn ModuleActivationReader>,
    writer: Arc<dyn ModuleActivationWriter>,
    organizations: Arc<dyn OrganizationReader>,
    registered: BTreeSet<ModuleKey>,
}

impl ModuleActivationService {
    pub fn new(
        reader: Arc<dyn ModuleActivationReader>,
        writer: Arc<dyn ModuleActivationWriter>,
        organizations: Arc<dyn OrganizationReader>,
        registered: BTreeSet<ModuleKey>,
    ) -> Self {
        Self {
            reader,
            writer,
            organizations,
            registered,
        }
    }

    pub fn registered(&self) -> &BTreeSet<ModuleKey> {
        &self.registered
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn resolution(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ActivationResolution>, ServiceError> {
        Ok(self.reader.resolve(org).await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn own(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, bool>, ServiceError> {
        Ok(self.reader.own(org).await?)
    }

    /// The lock check mirrors `SettingsService::update` but runs per registered
    /// module: a write into a frozen subtree fails even when it would change
    /// nothing, except with no module registered, where there is nothing to write.
    #[tracing::instrument(level = "info", skip(self, changes))]
    pub async fn update(
        &self,
        org: Id<Organization>,
        changes: BTreeMap<String, Option<bool>>,
    ) -> Result<(), ServiceError> {
        let mut parsed = BTreeMap::new();
        for (raw, change) in changes {
            let key = ModuleKey::new(&raw)
                .ok()
                .filter(|key| self.registered.contains(key))
                .ok_or(ServiceError::ModuleUnknown { key: raw })?;
            parsed.insert(key, change);
        }

        let resolution = self.reader.resolve(org).await?;
        if let Some(enforcer) = resolution.values().find_map(|r| r.enforced_by) {
            return Err(ServiceError::SettingsEnforcedByAncestor {
                organization_id: enforcer.value(),
            });
        }
        if parsed.is_empty() {
            return Ok(());
        }
        Ok(self.writer.apply(org, &parsed).await?)
    }

    /// Active modules per reachable organization. Organizations without any
    /// active module are listed with an empty set, so a client can tell
    /// "nothing active here" from "not reachable".
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn active_by_org(
        &self,
        reachable: &Visibility,
    ) -> Result<BTreeMap<Id<Organization>, BTreeSet<ModuleKey>>, ServiceError> {
        let orgs: Vec<Id<Organization>> = match reachable {
            Visibility::Unrestricted => self
                .organizations
                .all()
                .await?
                .into_iter()
                .map(|o| o.id)
                .collect(),
            Visibility::Only(ids) => ids.iter().copied().collect(),
        };
        let mut result: BTreeMap<Id<Organization>, BTreeSet<ModuleKey>> =
            orgs.into_iter().map(|id| (id, BTreeSet::new())).collect();
        for key in &self.registered {
            for org in self.reader.active_orgs(key).await? {
                if let Some(active) = result.get_mut(&org) {
                    active.insert(key.clone());
                }
            }
        }
        Ok(result)
    }
}
