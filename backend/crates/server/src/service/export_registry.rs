use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

use domain::{data_export::ExportCatalog, module::ModuleKey};

use crate::service::targets::RegistryAlreadyInstalled;

/// Filled once after the modules are built, like the statistics registry.
#[derive(Default)]
pub struct ExportRegistry {
    catalogs: OnceLock<BTreeMap<ModuleKey, Arc<dyn ExportCatalog>>>,
}

impl ExportRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(
        &self,
        catalogs: BTreeMap<ModuleKey, Arc<dyn ExportCatalog>>,
    ) -> Result<(), RegistryAlreadyInstalled> {
        self.catalogs
            .set(catalogs)
            .map_err(|_| RegistryAlreadyInstalled)
    }

    pub fn catalogs(&self) -> Vec<(ModuleKey, Arc<dyn ExportCatalog>)> {
        self.catalogs
            .get()
            .into_iter()
            .flatten()
            .map(|(key, catalog)| (key.clone(), catalog.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use domain::{
        Id, RepositoryError,
        data_export::{ExportCatalog, ExportThing},
        organization::Organization,
    };

    use super::*;

    struct Fake;

    #[async_trait::async_trait]
    impl ExportCatalog for Fake {
        async fn snapshot(
            &self,
            _: &BTreeSet<Id<Organization>>,
        ) -> Result<Vec<ExportThing>, RepositoryError> {
            Ok(vec![])
        }
    }

    fn registry() -> ExportRegistry {
        let r = ExportRegistry::new();
        let mut map: BTreeMap<ModuleKey, Arc<dyn ExportCatalog>> = BTreeMap::new();
        map.insert(ModuleKey::new("fake").unwrap(), Arc::new(Fake));
        r.install(map).unwrap();
        r
    }

    #[test]
    fn catalogs_carry_their_module_key() {
        let catalogs = registry().catalogs();
        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].0.as_str(), "fake");
    }

    #[test]
    fn an_uninstalled_registry_has_no_catalogs() {
        assert!(ExportRegistry::new().catalogs().is_empty());
    }

    #[test]
    fn a_registry_installs_only_once() {
        assert!(registry().install(BTreeMap::new()).is_err());
    }
}
