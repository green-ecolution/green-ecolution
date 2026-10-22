use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rustix::process::{Pid, Signal, kill_process_group};

use crate::sync::lock;

/// Process groups geco started, mirrored to `.geco/state.json` so the next run can find
/// survivors of a crash.
#[derive(Clone, Default)]
pub struct ProcessRegistry {
    groups: Arc<Mutex<BTreeSet<i32>>>,
    path: Option<PathBuf>,
}

impl ProcessRegistry {
    pub fn persistent(path: PathBuf) -> Self {
        Self {
            groups: Arc::default(),
            path: Some(path),
        }
    }

    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn insert(&self, group: i32) {
        let mut groups = lock(&self.groups);
        groups.insert(group);
        self.persist(&groups);
    }

    pub fn remove(&self, group: i32) {
        let mut groups = lock(&self.groups);
        groups.remove(&group);
        self.persist(&groups);
    }

    pub fn groups(&self) -> Vec<i32> {
        lock(&self.groups).iter().copied().collect()
    }

    /// Signals without waiting; meant for the panic hook.
    pub fn terminate_all_now(&self) {
        terminate_groups(&self.groups());
    }

    fn persist(&self, groups: &BTreeSet<i32>) {
        let Some(path) = &self.path else {
            return;
        };
        // A lost state file only weakens orphan detection; it must not stop a unit.
        if let Ok(json) = serde_json::to_vec(groups) {
            let _ = std::fs::write(path, json);
        }
    }
}

pub fn terminate_groups(groups: &[i32]) {
    for pid in groups.iter().filter_map(|group| Pid::from_raw(*group)) {
        let _ = kill_process_group(pid, Signal::TERM);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_groups_to_the_state_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let registry = ProcessRegistry::persistent(path.clone());

        registry.insert(42);
        registry.insert(7);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[7,42]");

        registry.remove(42);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[7]");
        assert_eq!(registry.groups(), [7]);
    }
}
