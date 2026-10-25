use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

use domain::{
    RepositoryError,
    module::{ModuleCount, ModuleKey, ModuleStatistics},
};

use crate::service::targets::RegistryAlreadyInstalled;

/// Filled once after the modules are built, like [`super::planning::PlanningRegistry`].
#[derive(Default)]
pub struct StatisticsRegistry {
    providers: OnceLock<BTreeMap<ModuleKey, Arc<dyn ModuleStatistics>>>,
}

impl StatisticsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(
        &self,
        providers: BTreeMap<ModuleKey, Arc<dyn ModuleStatistics>>,
    ) -> Result<(), RegistryAlreadyInstalled> {
        self.providers
            .set(providers)
            .map_err(|_| RegistryAlreadyInstalled)
    }

    pub async fn module_counts(&self) -> Result<Vec<(ModuleKey, ModuleCount)>, RepositoryError> {
        let mut all = Vec::new();
        for (key, provider) in self.providers.get().into_iter().flatten() {
            all.extend(
                provider
                    .counts()
                    .await?
                    .into_iter()
                    .map(|count| (key.clone(), count)),
            );
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;

    #[async_trait::async_trait]
    impl ModuleStatistics for Fake {
        async fn counts(&self) -> Result<Vec<ModuleCount>, RepositoryError> {
            Ok(vec![ModuleCount {
                kind: "thing",
                count: 3,
            }])
        }
    }

    fn registry() -> StatisticsRegistry {
        let r = StatisticsRegistry::new();
        let mut map: BTreeMap<ModuleKey, Arc<dyn ModuleStatistics>> = BTreeMap::new();
        map.insert(ModuleKey::new("fake").unwrap(), Arc::new(Fake));
        r.install(map).unwrap();
        r
    }

    #[tokio::test]
    async fn counts_carry_their_module_key() {
        let counts = registry().module_counts().await.unwrap();
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].0.as_str(), "fake");
        assert_eq!(counts[0].1.count, 3);
    }

    #[tokio::test]
    async fn an_uninstalled_registry_has_no_counts() {
        assert!(
            StatisticsRegistry::new()
                .module_counts()
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_registry_installs_only_once() {
        assert!(registry().install(BTreeMap::new()).is_err());
    }
}
