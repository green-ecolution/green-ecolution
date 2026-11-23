use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    authorization::Visibility,
    module::{ModuleActivationReader, ModuleActivationWriter, ModuleKey, ModuleState},
    organization::{Organization, OrganizationReader},
};

use crate::service::ServiceError;

pub struct ModuleListing {
    /// Modules released in at least one of the organizations asked for.
    pub released: BTreeSet<ModuleKey>,
    /// Active modules per organization; an organization without any is
    /// listed with an empty set, which tells "nothing active" from "not reachable".
    pub active: BTreeMap<Id<Organization>, BTreeSet<ModuleKey>>,
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
    pub async fn states(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ModuleState>, ServiceError> {
        Ok(self.reader.resolve(org).await?)
    }

    /// Whether `org` holds its own switches: a tenant or the root.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn switchable(&self, org: Id<Organization>) -> Result<bool, ServiceError> {
        // The service's own hierarchy, not the one of the access context: the
        // demo bypass carries an empty one.
        let hierarchy = self.organizations.hierarchy().await?;
        if !hierarchy.knows(org) {
            return Err(RepositoryError::NotFound.into());
        }
        Ok(hierarchy.tenant_of(org).is_none_or(|tenant| tenant == org))
    }

    /// Switches released modules of a tenant or the root. A locked module
    /// answers like an unknown one, so a client cannot probe what is locked.
    #[tracing::instrument(level = "info", skip(self, changes))]
    pub async fn update(
        &self,
        org: Id<Organization>,
        changes: BTreeMap<String, bool>,
    ) -> Result<(), ServiceError> {
        let mut parsed = BTreeMap::new();
        for (raw, enabled) in changes {
            let key = ModuleKey::new(&raw)
                .ok()
                .filter(|key| self.registered.contains(key))
                .ok_or(ServiceError::ModuleUnknown { key: raw })?;
            parsed.insert(key, enabled);
        }

        if parsed.is_empty() {
            return Ok(());
        }
        if !self.switchable(org).await? {
            return Err(ServiceError::ModuleNotSwitchable);
        }

        let states = self.states(org).await?;
        for key in parsed.keys() {
            if !states.get(key).is_some_and(|state| state.released) {
                return Err(ServiceError::ModuleUnknown {
                    key: key.to_string(),
                });
            }
        }
        parsed.retain(|key, enabled| states.get(key).map(|s| s.enabled) != Some(*enabled));
        if parsed.is_empty() {
            return Ok(());
        }
        Ok(self.writer.switch(org, &parsed).await?)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn listing(&self, reachable: &Visibility) -> Result<ModuleListing, ServiceError> {
        let orgs: BTreeSet<Id<Organization>> = match reachable {
            Visibility::Unrestricted => self
                .organizations
                .all()
                .await?
                .into_iter()
                .map(|o| o.id)
                .collect(),
            Visibility::Only(ids) => ids.iter().copied().collect(),
        };
        let mut active: BTreeMap<Id<Organization>, BTreeSet<ModuleKey>> =
            orgs.iter().map(|id| (*id, BTreeSet::new())).collect();
        let mut released = BTreeSet::new();
        for key in &self.registered {
            if !self.reader.released_orgs(key).await?.is_disjoint(&orgs) {
                released.insert(key.clone());
            }
            for org in self.reader.active_orgs(key).await? {
                if let Some(keys) = active.get_mut(&org) {
                    keys.insert(key.clone());
                }
            }
        }
        Ok(ModuleListing { released, active })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;
    use domain::authorization::OrgHierarchy;

    use super::*;

    const ON: ModuleState = ModuleState {
        released: true,
        enabled: true,
    };
    const LOCKED: ModuleState = ModuleState {
        released: false,
        enabled: false,
    };

    type Switched = Vec<(Id<Organization>, BTreeMap<ModuleKey, bool>)>;

    struct Fake {
        hierarchy: OrgHierarchy,
        states: Mutex<BTreeMap<ModuleKey, ModuleState>>,
        switched: Mutex<Switched>,
    }

    #[async_trait]
    impl ModuleActivationReader for Fake {
        async fn resolve(
            &self,
            _org: Id<Organization>,
        ) -> Result<BTreeMap<ModuleKey, ModuleState>, RepositoryError> {
            Ok(self.states.lock().unwrap().clone())
        }
        async fn active_orgs(
            &self,
            _key: &ModuleKey,
        ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
            Ok(BTreeSet::new())
        }
        async fn released_orgs(
            &self,
            _key: &ModuleKey,
        ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
            Ok(BTreeSet::new())
        }
        async fn tenants(
            &self,
        ) -> Result<BTreeMap<Id<Organization>, BTreeMap<ModuleKey, ModuleState>>, RepositoryError>
        {
            Ok(BTreeMap::new())
        }
    }

    #[async_trait]
    impl ModuleActivationWriter for Fake {
        async fn switch(
            &self,
            holder: Id<Organization>,
            changes: &BTreeMap<ModuleKey, bool>,
        ) -> Result<(), RepositoryError> {
            self.switched
                .lock()
                .unwrap()
                .push((holder, changes.clone()));
            Ok(())
        }
        async fn release(
            &self,
            _tenant: Id<Organization>,
            _changes: &BTreeMap<ModuleKey, bool>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[async_trait]
    impl OrganizationReader for Fake {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, _id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(self.hierarchy.clone())
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(HashMap::new())
        }
        async fn resource_counts(
            &self,
        ) -> Result<
            HashMap<Id<Organization>, domain::organization::OrganizationResourceCounts>,
            RepositoryError,
        > {
            Ok(HashMap::new())
        }
    }

    fn key(raw: &str) -> ModuleKey {
        ModuleKey::new(raw).unwrap()
    }

    struct Setup {
        fake: Arc<Fake>,
        service: ModuleActivationService,
        root: Id<Organization>,
        tenant: Id<Organization>,
        sub: Id<Organization>,
    }

    fn setup(states: &[(&str, ModuleState)]) -> Setup {
        let root = Id::new_v7();
        let tenant = Id::new_v7();
        let sub = Id::new_v7();
        let fake = Arc::new(Fake {
            hierarchy: OrgHierarchy::from_pairs([
                (root, None),
                (tenant, Some(root)),
                (sub, Some(tenant)),
            ]),
            states: Mutex::new(states.iter().map(|(k, s)| (key(k), *s)).collect()),
            switched: Mutex::new(Vec::new()),
        });
        let service = ModuleActivationService::new(
            fake.clone(),
            fake.clone(),
            fake.clone(),
            states.iter().map(|(k, _)| key(k)).collect(),
        );
        Setup {
            fake,
            service,
            root,
            tenant,
            sub,
        }
    }

    fn change(raw: &str, enabled: bool) -> BTreeMap<String, bool> {
        BTreeMap::from([(raw.to_string(), enabled)])
    }

    #[tokio::test]
    async fn a_sub_organization_cannot_switch() {
        let s = setup(&[("alpha", ON)]);
        let err = s.service.update(s.sub, change("alpha", false)).await;
        assert!(matches!(err, Err(ServiceError::ModuleNotSwitchable)));
        assert!(s.fake.switched.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_locked_module_answers_like_an_unknown_one() {
        let s = setup(&[("alpha", LOCKED)]);
        let err = s.service.update(s.tenant, change("alpha", true)).await;
        assert!(matches!(err, Err(ServiceError::ModuleUnknown { key }) if key == "alpha"));
        let err = s.service.update(s.tenant, change("ghost", true)).await;
        assert!(matches!(err, Err(ServiceError::ModuleUnknown { .. })));
    }

    #[tokio::test]
    async fn the_same_value_writes_nothing() {
        let s = setup(&[("alpha", ON)]);
        s.service
            .update(s.tenant, change("alpha", true))
            .await
            .unwrap();
        assert!(s.fake.switched.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_changed_value_is_written_for_the_tenant_and_the_root() {
        let s = setup(&[("alpha", ON)]);
        s.service
            .update(s.tenant, change("alpha", false))
            .await
            .unwrap();
        s.service
            .update(s.root, change("alpha", false))
            .await
            .unwrap();
        let switched = s.fake.switched.lock().unwrap();
        assert_eq!(switched.len(), 2);
        assert_eq!(switched[0].0, s.tenant);
        assert_eq!(switched[1].0, s.root);
        assert_eq!(switched[0].1, BTreeMap::from([(key("alpha"), false)]));
    }

    #[tokio::test]
    async fn an_unknown_organization_is_not_found() {
        let s = setup(&[("alpha", ON)]);
        let err = s.service.switchable(Id::new_v7()).await;
        assert!(matches!(
            err,
            Err(ServiceError::Repository(RepositoryError::NotFound))
        ));
    }
}
