use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    assemble::{AssembleError, assemble},
    config::{ConfigError, GecoConfig},
    paths::Paths,
    state::ProcessRegistry,
    supervisor::{ProfileChange, Supervisor, SupervisorError},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileList {
    pub active: String,
    pub available: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Assemble(#[from] AssembleError),
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
}

pub struct Profiles {
    paths: Paths,
    registry: ProcessRegistry,
    supervisor: Arc<Supervisor>,
}

impl Profiles {
    pub fn new(root: PathBuf, registry: ProcessRegistry, supervisor: Arc<Supervisor>) -> Self {
        Self {
            paths: Paths::new(root),
            registry,
            supervisor,
        }
    }

    pub fn list(&self) -> Result<ProfileList, ProfileError> {
        let config = GecoConfig::load(&self.paths.config())?;
        Ok(ProfileList {
            active: self.supervisor.profile(),
            available: config.profile.keys().cloned().collect(),
        })
    }

    /// Reads geco.toml again, so an edited profile takes effect on the next switch to it.
    /// Choosing the active profile again changes nothing.
    pub async fn switch(&self, name: &str) -> Result<ProfileChange, ProfileError> {
        if self.supervisor.profile() == name {
            return Ok(ProfileChange {
                profile: name.to_owned(),
                warnings: vec![format!("already on profile {name}")],
                ..ProfileChange::default()
            });
        }
        let config = GecoConfig::load(&self.paths.config())?;
        let assembly = assemble(&self.paths.root, &config, name, self.registry.clone()).await?;
        let mut change = self
            .supervisor
            .apply(name, assembly.units, assembly.graph)
            .await?;
        change.warnings = assembly.warnings;
        Ok(change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::Graph,
        logs::LogStore,
        units::{
            Unit,
            fake::{FakeUnit, Journal},
        },
    };

    #[tokio::test]
    async fn switching_to_the_active_profile_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("geco.toml"),
            "[profile.minimal]\nunits = []\n",
        )
        .unwrap();
        let journal = Journal::default();
        let units: Vec<Arc<dyn Unit>> = vec![Arc::new(FakeUnit::service("db", &journal))];
        let graph = Graph::new([("db".to_owned(), Vec::<String>::new())]).unwrap();
        let supervisor = Supervisor::new(units, graph, LogStore::new(100));
        supervisor.set_profile("minimal");
        let profiles = Profiles::new(
            dir.path().to_path_buf(),
            ProcessRegistry::in_memory(),
            Arc::clone(&supervisor),
        );

        let change = profiles.switch("minimal").await.unwrap();

        assert_eq!(
            change,
            ProfileChange {
                profile: "minimal".into(),
                warnings: vec!["already on profile minimal".into()],
                ..ProfileChange::default()
            }
        );
        assert_eq!(
            supervisor.status().len(),
            1,
            "the profile was applied again"
        );
        assert!(journal.entries().is_empty());
    }
}
