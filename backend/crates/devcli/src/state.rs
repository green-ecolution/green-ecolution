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

use serde::{Deserialize, Serialize};

use crate::sync::lock;

/// Process groups geco started, mirrored to `.geco/state.json` so the next run can find
/// survivors of a crash.
#[derive(Clone, Default)]
pub struct ProcessRegistry {
    groups: Arc<Mutex<BTreeSet<i32>>>,
    path: Option<PathBuf>,
    boot_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    boot_id: Option<String>,
    groups: BTreeSet<i32>,
}

impl ProcessRegistry {
    pub fn persistent(path: PathBuf) -> Self {
        Self {
            groups: Arc::default(),
            path: Some(path),
            boot_id: current_boot_id(),
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
        let state = StateFile {
            boot_id: self.boot_id.clone(),
            groups: groups.clone(),
        };
        // A lost state file only weakens orphan detection; it must not stop a unit.
        if let Ok(json) = serde_json::to_vec(&state) {
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
    orphans_for_boot(path, current_boot_id().as_deref())
}

fn orphans_for_boot(path: &Path, boot_id: Option<&str>) -> Vec<i32> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    // A file in an older format lists no boot id, so its groups cannot be trusted either.
    let Ok(state) = serde_json::from_str::<StateFile>(&text) else {
        return Vec::new();
    };
    // After a reboot the recorded numbers may belong to unrelated process groups.
    if let (Some(recorded), Some(current)) = (state.boot_id.as_deref(), boot_id)
        && recorded != current
    {
        return Vec::new();
    }
    state
        .groups
        .into_iter()
        .filter(|group| {
            Pid::from_raw(*group).is_some_and(|pid| test_kill_process_group(pid).is_ok())
        })
        .collect()
}

/// Changes with every boot; `None` when the platform offers neither source.
pub(crate) fn current_boot_id() -> Option<String> {
    if let Ok(id) = fs::read_to_string("/proc/sys/kernel/random/boot_id") {
        return Some(id.trim().to_owned());
    }
    let output = std::process::Command::new("sysctl")
        .args(["-n", "kern.boottime"])
        .output()
        .ok()?;
    let id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (output.status.success() && !id.is_empty()).then_some(id)
}

/// The command name of the group's leader; `None` once the leader has exited.
pub fn group_leader_name(group: i32) -> Option<String> {
    let output = std::process::Command::new("ps")
        .args(["-o", "comm=", "-p", &group.to_string()])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (output.status.success() && !name.is_empty()).then_some(name)
}

/// `1234 (bacon), 5678 (leader exited)`, for asking whether to stop them.
pub fn describe_groups(groups: &[i32]) -> String {
    groups
        .iter()
        .map(|group| match group_leader_name(*group) {
            Some(name) => format!("{group} ({name})"),
            None => format!("{group} (leader exited)"),
        })
        .collect::<Vec<_>>()
        .join(", ")
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
        assert_eq!(read_state(&path).groups, BTreeSet::from([7, 42]));

        registry.remove(42);
        let state = read_state(&path);
        assert_eq!(state.groups, BTreeSet::from([7]));
        assert_eq!(state.boot_id, current_boot_id());
        assert_eq!(registry.groups(), [7]);
    }

    fn read_state(path: &Path) -> StateFile {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn write_state(path: &Path, boot_id: Option<&str>, groups: &[i32]) {
        let state = StateFile {
            boot_id: boot_id.map(str::to_owned),
            groups: groups.iter().copied().collect(),
        };
        std::fs::write(path, serde_json::to_vec(&state).unwrap()).unwrap();
    }

    fn own_group() -> i32 {
        rustix::process::getpgrp().as_raw_nonzero().get()
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
        write_state(&path, Some("boot-a"), &[own_group(), i32::MAX]);

        assert_eq!(orphans_for_boot(&path, Some("boot-a")), [own_group()]);
        assert!(orphaned_groups(&dir.path().join("missing.json")).is_empty());
    }

    #[test]
    fn groups_recorded_before_a_reboot_are_no_orphans() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        write_state(&path, Some("boot-a"), &[own_group()]);

        assert!(orphans_for_boot(&path, Some("boot-b")).is_empty());
    }

    #[test]
    fn an_unknown_boot_id_on_either_side_keeps_the_groups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        write_state(&path, None, &[own_group()]);
        assert_eq!(orphans_for_boot(&path, Some("boot-a")), [own_group()]);

        write_state(&path, Some("boot-a"), &[own_group()]);
        assert_eq!(orphans_for_boot(&path, None), [own_group()]);
    }

    #[test]
    fn the_current_boot_is_recognised() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        write_state(&path, current_boot_id().as_deref(), &[own_group()]);

        assert_eq!(orphaned_groups(&path), [own_group()]);
    }

    #[test]
    fn the_old_bare_list_format_lists_no_orphans() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        std::fs::write(&path, format!("[{}]", own_group())).unwrap();

        assert!(orphaned_groups(&path).is_empty());
    }

    #[test]
    fn describes_groups_by_their_leader() {
        let own_pid = rustix::process::getpid().as_raw_nonzero().get();
        let described = describe_groups(&[own_pid, i32::MAX]);

        let name = group_leader_name(own_pid).unwrap();
        assert_eq!(
            described,
            format!("{own_pid} ({name}), {} (leader exited)", i32::MAX)
        );
    }
}
