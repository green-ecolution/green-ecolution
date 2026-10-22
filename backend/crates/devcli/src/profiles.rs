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

    /// Reads geco.toml again, so an edited profile takes effect on the next switch.
    pub async fn switch(&self, name: &str) -> Result<ProfileChange, ProfileError> {
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
