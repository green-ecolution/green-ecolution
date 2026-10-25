use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, OnceLock},
};

use domain::{
    Id, RepositoryError,
    authorization::ResourceKey,
    module::{ModuleActivationReader, ModuleKey},
    organization::Organization,
    shared::coordinates::Coordinate,
    target::{TargetCatalog, TargetError, TargetPurpose, TargetRef},
};

use crate::service::ServiceError;

#[derive(Debug, thiserror::Error)]
#[error("the target registry is already installed")]
pub struct RegistryAlreadyInstalled;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Admitted {
    pub organization_id: Id<Organization>,
    pub resource: ResourceKey,
}

/// Services are built before modules, modules provide the catalogs. Like the
/// deferred event bus, this handle is passed around first and filled once.
#[derive(Default)]
pub struct TargetRegistry {
    catalogs: OnceLock<BTreeMap<ModuleKey, Arc<dyn TargetCatalog>>>,
}

impl TargetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(
        &self,
        catalogs: BTreeMap<ModuleKey, Arc<dyn TargetCatalog>>,
    ) -> Result<(), RegistryAlreadyInstalled> {
        self.catalogs
            .set(catalogs)
            .map_err(|_| RegistryAlreadyInstalled)
    }

    fn catalog(&self, module: &ModuleKey) -> Option<&Arc<dyn TargetCatalog>> {
        self.catalogs.get().and_then(|c| c.get(module))
    }

    pub async fn admit(
        &self,
        target: &TargetRef,
        purpose: TargetPurpose,
    ) -> Result<Admitted, ServiceError> {
        let catalog = self
            .catalog(&target.module)
            .ok_or_else(|| ServiceError::ModuleUnknown {
                key: target.module.to_string(),
            })?;
        let spec = catalog
            .kinds()
            .iter()
            .find(|s| s.kind == target.kind.as_str() && s.allows(purpose))
            .ok_or_else(|| ServiceError::TargetNotAccepted {
                module: target.module.to_string(),
                kind: target.kind.to_string(),
            })?;
        let organization_id = catalog.admit(target, purpose).await.map_err(|e| match e {
            TargetError::NotFound => ServiceError::Repository(RepositoryError::NotFound),
            TargetError::Refused { code, message } => ServiceError::TargetRefused { code, message },
            TargetError::Repository(e) => e.into(),
        })?;
        Ok(Admitted {
            organization_id,
            resource: spec.resource,
        })
    }

    /// The resource permissions on `target` are checked against, whatever the
    /// target is used for. `None` for a module or kind no catalog knows.
    pub fn resource(&self, target: &TargetRef) -> Option<ResourceKey> {
        self.catalog(&target.module)?
            .kinds()
            .iter()
            .find(|s| s.kind == target.kind.as_str())
            .map(|s| s.resource)
    }

    pub async fn locate_all(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, Coordinate>, ServiceError> {
        let mut out = HashMap::new();
        for (module, group) in group_by_module(targets) {
            if let Some(catalog) = self.catalog(&module) {
                out.extend(catalog.locate(&group).await?);
            }
        }
        Ok(out)
    }

    pub async fn label_all(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, String>, ServiceError> {
        let mut out = HashMap::new();
        for (module, group) in group_by_module(targets) {
            if let Some(catalog) = self.catalog(&module) {
                out.extend(catalog.label(&group).await?);
            }
        }
        Ok(out)
    }
}

pub async fn ensure_module_active(
    activation: &dyn ModuleActivationReader,
    module: &ModuleKey,
    org: Id<Organization>,
) -> Result<(), ServiceError> {
    let active = activation
        .resolve(org)
        .await?
        .get(module)
        .is_some_and(|r| r.enabled);
    if active {
        Ok(())
    } else {
        Err(ServiceError::ModuleDisabled {
            module: module.to_string(),
        })
    }
}

fn group_by_module(targets: &[TargetRef]) -> BTreeMap<ModuleKey, Vec<TargetRef>> {
    let mut groups: BTreeMap<ModuleKey, Vec<TargetRef>> = BTreeMap::new();
    for t in targets {
        groups.entry(t.module.clone()).or_default().push(t.clone());
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::target::TargetKindSpec;
    use uuid::Uuid;

    struct Fake;

    const SPECS: [TargetKindSpec; 1] = [TargetKindSpec {
        kind: "thing",
        accepts_sensors: true,
        commentable: false,
        resource: ResourceKey::declare("thing"),
    }];

    #[async_trait::async_trait]
    impl TargetCatalog for Fake {
        fn kinds(&self) -> &'static [TargetKindSpec] {
            &SPECS
        }
        async fn admit(
            &self,
            target: &TargetRef,
            _purpose: TargetPurpose,
        ) -> Result<Id<Organization>, TargetError> {
            if target.id.is_nil() {
                Err(TargetError::NotFound)
            } else {
                Ok(Id::new(target.id))
            }
        }
        async fn locate(
            &self,
            targets: &[TargetRef],
        ) -> Result<HashMap<TargetRef, Coordinate>, RepositoryError> {
            Ok(targets
                .iter()
                .map(|t| (t.clone(), Coordinate::new(54.0, 9.0).unwrap()))
                .collect())
        }
        async fn label(
            &self,
            targets: &[TargetRef],
        ) -> Result<HashMap<TargetRef, String>, RepositoryError> {
            Ok(targets
                .iter()
                .map(|t| (t.clone(), "x".to_string()))
                .collect())
        }
    }

    fn registry() -> TargetRegistry {
        let r = TargetRegistry::new();
        let mut map: BTreeMap<ModuleKey, Arc<dyn TargetCatalog>> = BTreeMap::new();
        map.insert(ModuleKey::new("fake").unwrap(), Arc::new(Fake));
        r.install(map).unwrap();
        r
    }

    fn target(module: &str, kind: &str, id: Uuid) -> TargetRef {
        TargetRef::new(module, kind, id).unwrap()
    }

    #[tokio::test]
    async fn admits_a_known_kind_for_its_purpose() {
        let id = Uuid::now_v7();
        let a = registry()
            .admit(&target("fake", "thing", id), TargetPurpose::Sensor)
            .await
            .unwrap();
        assert_eq!(a.organization_id, Id::new(id));
        assert_eq!(a.resource, ResourceKey::declare("thing"));
    }

    #[tokio::test]
    async fn unknown_module_is_module_unknown() {
        let r = registry()
            .admit(
                &target("ghost", "thing", Uuid::now_v7()),
                TargetPurpose::Sensor,
            )
            .await;
        assert!(matches!(r, Err(ServiceError::ModuleUnknown { .. })));
    }

    #[tokio::test]
    async fn wrong_kind_or_purpose_is_not_accepted() {
        let reg = registry();
        let r = reg
            .admit(
                &target("fake", "other", Uuid::now_v7()),
                TargetPurpose::Sensor,
            )
            .await;
        assert!(matches!(r, Err(ServiceError::TargetNotAccepted { .. })));
        let r = reg
            .admit(
                &target("fake", "thing", Uuid::now_v7()),
                TargetPurpose::Comment,
            )
            .await;
        assert!(matches!(r, Err(ServiceError::TargetNotAccepted { .. })));
    }

    #[tokio::test]
    async fn missing_target_is_not_found() {
        let r = registry()
            .admit(&target("fake", "thing", Uuid::nil()), TargetPurpose::Sensor)
            .await;
        assert!(matches!(
            r,
            Err(ServiceError::Repository(RepositoryError::NotFound))
        ));
    }

    #[test]
    fn resource_is_looked_up_by_kind() {
        let reg = registry();
        assert_eq!(
            reg.resource(&target("fake", "thing", Uuid::now_v7())),
            Some(ResourceKey::declare("thing"))
        );
        assert_eq!(reg.resource(&target("fake", "other", Uuid::now_v7())), None);
        assert_eq!(
            reg.resource(&target("ghost", "thing", Uuid::now_v7())),
            None
        );
    }

    #[tokio::test]
    async fn locate_all_groups_by_module_and_skips_unknown_modules() {
        let reg = registry();
        let known = target("fake", "thing", Uuid::now_v7());
        let unknown = target("ghost", "thing", Uuid::now_v7());
        let located = reg.locate_all(&[known.clone(), unknown]).await.unwrap();
        assert_eq!(located.len(), 1);
        assert!(located.contains_key(&known));
    }
}
