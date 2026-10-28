use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use domain::{
    Id,
    authorization::{AccessContext, Action, Permission, ResourceKey, Visibility},
    module::{
        ActivationResolution, ModuleActivationReader, ModuleActivationWriter, ModuleKey, OwnSwitch,
    },
    organization::{Organization, OrganizationReader},
};

use crate::service::{AuthError, ServiceError};

fn setting_update() -> Permission {
    Permission::new(ResourceKey::SETTING, Action::Update)
}

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
    ) -> Result<BTreeMap<ModuleKey, OwnSwitch>, ServiceError> {
        Ok(self.reader.own(org).await?)
    }

    /// Whether `ctx` may change a switch set from `set_by`.
    pub fn may_change(ctx: &AccessContext, switch: &OwnSwitch) -> bool {
        ctx.allows_in(setting_update(), switch.set_by)
    }

    /// The lock check mirrors `SettingsService::update` but runs per registered
    /// module: a write into a frozen subtree fails even when it would change
    /// nothing, except with no module registered, where there is nothing to write.
    ///
    /// A switch is written as the topmost organization on the chain the caller
    /// may update settings for, so a parent's choice binds the sub-unit. A
    /// switch set from a level the caller does not act for is refused; resending
    /// the stored value is a no-op, otherwise a sub-unit could take over a
    /// parent's choice by repeating it.
    #[tracing::instrument(level = "info", skip(self, ctx, changes))]
    pub async fn update(
        &self,
        ctx: &AccessContext,
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

        let own = self.reader.own(org).await?;
        parsed.retain(|key, change| own.get(key).map(|s| s.enabled) != *change);
        if let Some(foreign) = parsed
            .keys()
            .filter_map(|key| own.get(key))
            .find(|switch| !Self::may_change(ctx, switch))
        {
            return Err(ServiceError::SettingsSetByAncestor {
                organization_id: foreign.set_by.value(),
            });
        }
        if parsed.is_empty() {
            return Ok(());
        }

        // The service's own hierarchy, not `ctx.hierarchy`: the demo bypass
        // carries an empty one and would always act as `org` itself.
        let acting = self
            .organizations
            .hierarchy()
            .await?
            .ancestors_from_root(org)
            .into_iter()
            .find(|level| ctx.allows_in(setting_update(), *level))
            .ok_or(AuthError::Forbidden)?;
        Ok(self.writer.apply(org, &parsed, acting).await?)
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
