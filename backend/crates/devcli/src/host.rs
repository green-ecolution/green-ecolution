use std::{
    fs,
    io::{self, BufRead, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::Arc,
};

use tokio::{
    net::UnixListener,
    signal::unix::{SignalKind, signal},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{
    assemble::{AssembleError, assemble},
    config::{ConfigError, GecoConfig, JustVars},
    doctor::{Env, checks, service::Doctor},
    ipc,
    logs::LogStore,
    paths::Paths,
    state::{InstanceLock, ProcessRegistry, describe_groups, orphaned_groups, stop_orphans},
    supervisor::Supervisor,
    units::{
        compose::ComposeCli,
        process::{STOP_GRACE, terminate_registered},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrphanPolicy {
    Ask,
    Abort,
}

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("a geco supervisor is already running for this repository")]
    AlreadyRunning,
    #[error(
        "process groups {0:?} from an earlier geco run are still alive; run `geco up` in a terminal to stop them"
    )]
    Orphans(Vec<i32>),
    #[error("aborted")]
    Aborted,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Assemble(#[from] AssembleError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Host {
    pub supervisor: Arc<Supervisor>,
    pub shutdown: CancellationToken,
    pub registry: ProcessRegistry,
    pub warnings: Vec<String>,
    pub doctor: Arc<Doctor>,
    server: JoinHandle<()>,
    watcher: JoinHandle<()>,
    socket: PathBuf,
    _lock: InstanceLock,
}

impl Host {
    pub async fn start(
        paths: &Paths,
        profile: Option<&str>,
        orphans: OrphanPolicy,
    ) -> Result<Self, HostError> {
        let lock = InstanceLock::try_acquire(&paths.lock())?.ok_or(HostError::AlreadyRunning)?;
        handle_orphans(paths, orphans).await?;
        let config = GecoConfig::load(&paths.config())?;
        let profile = profile.unwrap_or(&config.default_profile);
        let registry = ProcessRegistry::persistent(paths.process_state());
        let assembly = assemble(&paths.root, &config, profile, registry.clone()).await?;
        let supervisor = Supervisor::new(
            assembly.units,
            assembly.graph,
            LogStore::new(config.logs.capacity),
        );
        let doctor = Doctor::new(
            checks::all(),
            Env {
                root: paths.root.clone(),
                compose: ComposeCli::new(paths.root.clone()),
                vars: Arc::new(JustVars::new(paths.root.clone())),
                units: Vec::new(),
            },
            Arc::clone(&supervisor),
            registry.clone(),
        );

        let socket = paths.socket();
        // Holding the lock proves that a leftover socket file belongs to a dead supervisor.
        match fs::remove_file(&socket) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err.into()),
            _ => {}
        }
        let listener = UnixListener::bind(&socket)?;
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;

        let shutdown = CancellationToken::new();
        let watcher = doctor.watch(shutdown.clone());
        let server = tokio::spawn(ipc::server::serve(
            listener,
            Arc::clone(&supervisor),
            Arc::clone(&doctor),
            shutdown.clone(),
        ));
        supervisor.start_all().await;
        let startup = Arc::clone(&doctor);
        tokio::spawn(async move {
            startup.run(false).await;
        });
        Ok(Self {
            supervisor,
            shutdown,
            registry,
            warnings: assembly.warnings,
            doctor,
            server,
            watcher,
            socket,
            _lock: lock,
        })
    }

    /// Resolves on `geco down`, Ctrl-C, SIGTERM or SIGHUP.
    pub async fn wait(&self) {
        tokio::select! {
            () = self.shutdown.cancelled() => {}
            () = termination() => {}
        }
    }

    pub async fn stop(self) {
        self.shutdown.cancel();
        self.supervisor.stop_all().await;
        terminate_registered(&self.registry, STOP_GRACE).await;
        self.server.abort();
        self.watcher.abort();
        let _ = fs::remove_file(&self.socket);
    }
}

/// SIGHUP arrives when the terminal closes; the units sit in their own process groups and
/// would survive geco dying from it.
pub async fn termination() {
    tokio::select! {
        () = received(SignalKind::interrupt()) => {}
        () = received(SignalKind::terminate()) => {}
        () = received(SignalKind::hangup()) => {}
    }
}

/// A signal that cannot be registered never arrives.
async fn received(kind: SignalKind) {
    match signal(kind) {
        Ok(mut stream) => {
            stream.recv().await;
        }
        Err(_) => futures::future::pending::<()>().await,
    }
}

async fn handle_orphans(paths: &Paths, policy: OrphanPolicy) -> Result<(), HostError> {
    let groups = orphaned_groups(&paths.process_state());
    if groups.is_empty() {
        return Ok(());
    }
    if policy == OrphanPolicy::Abort {
        return Err(HostError::Orphans(groups));
    }
    if !confirm(&orphan_prompt(&groups))? {
        return Err(HostError::Aborted);
    }
    stop_orphans(&paths.process_state(), &groups).await;
    Ok(())
}

pub fn orphan_prompt(groups: &[i32]) -> String {
    format!(
        "Process groups {} from an earlier geco run are still alive. Stop them? [y/N] ",
        describe_groups(groups)
    )
}

pub fn confirm(prompt: &str) -> io::Result<bool> {
    let mut stderr = io::stderr();
    stderr.write_all(prompt.as_bytes())?;
    stderr.flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rustix::process::Signal;

    use super::*;

    fn paths() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().to_path_buf());
        (dir, paths)
    }

    #[tokio::test]
    async fn a_hangup_ends_termination() {
        let mut terminated = std::pin::pin!(termination());
        // The first poll registers the handlers; a hangup before that would kill the test run.
        let early = tokio::time::timeout(Duration::from_millis(50), &mut terminated).await;
        assert!(early.is_err());

        rustix::process::kill_process(rustix::process::getpid(), Signal::HUP).unwrap();

        let ended = tokio::time::timeout(Duration::from_secs(5), terminated).await;
        assert!(ended.is_ok(), "SIGHUP did not end termination()");
    }

    #[tokio::test]
    async fn refuses_to_start_a_second_supervisor() {
        let (_dir, paths) = paths();
        let _held = InstanceLock::try_acquire(&paths.lock()).unwrap().unwrap();

        let result = Host::start(&paths, None, OrphanPolicy::Abort).await;

        assert!(matches!(result, Err(HostError::AlreadyRunning)));
    }

    #[tokio::test]
    async fn aborts_on_orphans_when_it_cannot_ask() {
        let (_dir, paths) = paths();
        fs::create_dir_all(paths.state_dir()).unwrap();
        let own_group = rustix::process::getpgrp().as_raw_nonzero().get();
        let state = serde_json::json!({
            "boot_id": crate::state::current_boot_id(),
            "groups": [own_group],
        });
        fs::write(paths.process_state(), state.to_string()).unwrap();

        let result = Host::start(&paths, None, OrphanPolicy::Abort).await;

        assert!(matches!(result, Err(HostError::Orphans(groups)) if groups == [own_group]));
    }
}
