use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rustix::{
    fs::{FlockOperation, flock},
    io::Errno,
    process::{Pid, Signal, kill_process_group, test_kill_process_group},
};

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

/// Held for the supervisor's whole lifetime; the kernel drops the flock when the process dies.
pub struct InstanceLock {
    _file: File,
}

impl InstanceLock {
    /// `None` when another supervisor already holds the lock for this repository.
    pub fn try_acquire(path: &Path) -> io::Result<Option<Self>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        match flock(&file, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(err) if err == Errno::WOULDBLOCK => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

/// Groups an earlier run recorded that still exist. Only meaningful while holding the lock,
/// otherwise a running supervisor's own groups would show up here.
pub fn orphaned_groups(path: &Path) -> Vec<i32> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let groups: Vec<i32> = serde_json::from_str(&text).unwrap_or_default();
    groups
        .into_iter()
        .filter(|group| {
            Pid::from_raw(*group).is_some_and(|pid| test_kill_process_group(pid).is_ok())
        })
        .collect()
}

/// Removing the file keeps a recycled process-group id from being offered again later.
pub fn stop_orphans(path: &Path, groups: &[i32]) {
    terminate_groups(groups);
    let _ = fs::remove_file(path);
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

    #[test]
    fn the_instance_lock_is_exclusive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".geco/lock");

        let first = InstanceLock::try_acquire(&path).unwrap();
        assert!(first.is_some());
        assert!(InstanceLock::try_acquire(&path).unwrap().is_none());

        drop(first);
        // A parallel test may fork while the fd is open; the child keeps the lock until it execs.
        let reacquired = (0..200).any(|_| {
            InstanceLock::try_acquire(&path).unwrap().is_some() || {
                std::thread::sleep(std::time::Duration::from_millis(10));
                false
            }
        });
        assert!(reacquired);
    }

    #[test]
    fn orphans_are_the_listed_groups_that_still_exist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let own_group = rustix::process::getpgrp().as_raw_nonzero().get();
        std::fs::write(&path, format!("[{own_group},{}]", i32::MAX)).unwrap();

        assert_eq!(orphaned_groups(&path), [own_group]);
        assert!(orphaned_groups(&dir.path().join("missing.json")).is_empty());
    }
}
