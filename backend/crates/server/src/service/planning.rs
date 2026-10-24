use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

use domain::{module::ModuleKey, planning::PlanningProvider};

use crate::service::{ServiceError, targets::RegistryAlreadyInstalled};

/// Filled once after the modules are built, like [`super::targets::TargetRegistry`].
#[derive(Default)]
pub struct PlanningRegistry {
    providers: OnceLock<BTreeMap<ModuleKey, Arc<dyn PlanningProvider>>>,
}

impl PlanningRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(
        &self,
        providers: BTreeMap<ModuleKey, Arc<dyn PlanningProvider>>,
    ) -> Result<(), RegistryAlreadyInstalled> {
        self.providers
            .set(providers)
            .map_err(|_| RegistryAlreadyInstalled)
    }

    pub fn provider(&self, module: &ModuleKey) -> Result<Arc<dyn PlanningProvider>, ServiceError> {
        self.providers
            .get()
            .and_then(|p| p.get(module))
            .cloned()
            .ok_or_else(|| ServiceError::ModuleUnknown {
                key: module.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        Id,
        authorization::ResourceKey,
        capacity::CapacityKind,
        organization::Organization,
        planning::{PlanningError, ResolvedStop, StopRef},
    };

    struct Fake;

    #[async_trait::async_trait]
    impl PlanningProvider for Fake {
        fn capacity_kind(&self) -> CapacityKind {
            CapacityKind::Water
        }

        fn plan_resource(&self) -> ResourceKey {
            ResourceKey::declare("fake_plan")
        }

        async fn resolve_stops(
            &self,
            _stops: &[StopRef],
            _org: Id<Organization>,
        ) -> Result<Vec<ResolvedStop>, PlanningError> {
            Ok(vec![])
        }
    }

    fn registry() -> PlanningRegistry {
        let r = PlanningRegistry::new();
        let mut map: BTreeMap<ModuleKey, Arc<dyn PlanningProvider>> = BTreeMap::new();
        map.insert(ModuleKey::new("fake").unwrap(), Arc::new(Fake));
        r.install(map).unwrap();
        r
    }

    #[test]
    fn a_known_module_has_its_provider() {
        let provider = registry()
            .provider(&ModuleKey::new("fake").unwrap())
            .unwrap();
        assert_eq!(provider.plan_resource(), ResourceKey::declare("fake_plan"));
    }

    #[test]
    fn an_unknown_module_is_module_unknown() {
        let r = registry().provider(&ModuleKey::new("ghost").unwrap());
        assert!(matches!(r, Err(ServiceError::ModuleUnknown { .. })));
    }

    #[test]
    fn a_registry_installs_only_once() {
        let r = registry();
        assert!(r.install(BTreeMap::new()).is_err());
    }
}
