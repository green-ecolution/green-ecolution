use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "geco.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn discover(start: &Path) -> Option<Self> {
        start
            .ancestors()
            .find(|dir| dir.join(CONFIG_FILE).is_file())
            .map(|dir| Self::new(dir.to_path_buf()))
    }

    pub fn config(&self) -> PathBuf {
        self.root.join(CONFIG_FILE)
    }

    pub fn state_dir(&self) -> PathBuf {
        self.root.join(".geco")
    }

    pub fn socket(&self) -> PathBuf {
        self.state_dir().join("geco.sock")
    }

    pub fn lock(&self) -> PathBuf {
        self.state_dir().join("lock")
    }

    pub fn process_state(&self) -> PathBuf {
        self.state_dir().join("state.json")
    }

    pub fn supervisor_log(&self) -> PathBuf {
        self.state_dir().join("supervisor.log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_the_root_from_a_nested_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), "").unwrap();
        let nested = dir.path().join("backend/crates");
        std::fs::create_dir_all(&nested).unwrap();

        let paths = Paths::discover(&nested).unwrap();

        assert_eq!(paths.root, dir.path());
        assert_eq!(paths.socket(), dir.path().join(".geco/geco.sock"));
    }

    #[test]
    fn returns_none_without_a_config_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Paths::discover(dir.path()).is_none());
    }
}
