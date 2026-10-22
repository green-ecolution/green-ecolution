use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use rustix::{
    io::Errno,
    process::{Pid, Signal, kill_process_group, test_kill_process_group},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::Command,
    sync::oneshot,
    task::JoinHandle,
    time::Instant,
};

use super::{Origin, Running, Unit, UnitError, UnitKind, probe};
use crate::{
    config::{LogFormat, ReadyProbe},
    logs::{LogSink, LogStore},
    state::ProcessRegistry,
    sync::lock,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSpec {
    pub name: String,
    pub kind: UnitKind,
    pub command: String,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub ready: Option<ReadyProbe>,
    pub log_format: LogFormat,
}

pub struct ProcessUnit {
    spec: ProcessSpec,
    registry: ProcessRegistry,
    grace: Duration,
    probe_interval: Duration,
    group: Mutex<Option<i32>>,
}

type ExitResult = Result<ExitStatus, String>;

impl ProcessUnit {
    pub fn new(spec: ProcessSpec, registry: ProcessRegistry) -> Self {
        Self {
            spec,
            registry,
            grace: Duration::from_secs(10),
            probe_interval: Duration::from_millis(500),
            group: Mutex::new(None),
        }
    }

    pub fn with_grace(mut self, grace: Duration) -> Self {
        self.grace = grace;
        self
    }
}

#[async_trait]
impl Unit for ProcessUnit {
    fn name(&self) -> &str {
        &self.spec.name
    }

    fn kind(&self) -> UnitKind {
        self.spec.kind
    }

    fn origin(&self) -> Origin {
        Origin::Local
    }

    async fn start(&self, logs: Arc<LogStore>) -> Result<Running, UnitError> {
        let sink = LogSink::new(logs, &self.spec.name, self.spec.log_format);
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.spec.command)
            .current_dir(&self.spec.cwd)
            .envs(&self.spec.env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // An own process group lets stop() reach everything bacon or pnpm spawn.
            .process_group(0)
            .spawn()
            .map_err(|source| UnitError::Spawn {
                command: self.spec.command.clone(),
                source,
            })?;
        if let Some(group) = child.id().and_then(|id| i32::try_from(id).ok()) {
            self.registry.insert(group);
            *lock(&self.group) = Some(group);
        }
        let readers = [
            forward_lines(child.stdout.take(), sink.clone()),
            forward_lines(child.stderr.take(), sink),
        ];
        let (exit_tx, mut exit_rx) = oneshot::channel::<ExitResult>();
        tokio::spawn(async move {
            let _ = exit_tx.send(child.wait().await.map_err(|err| err.to_string()));
        });

        match self.spec.kind {
            UnitKind::Oneshot => {
                let result = exit_rx.await.unwrap_or_else(|_| Err(lost()));
                // Dependents must see the complete output; a backgrounded grandchild may keep
                // the pipes open, hence the timeout.
                let readers = futures::future::join_all(readers.into_iter().flatten());
                let _ = tokio::time::timeout(Duration::from_secs(2), readers).await;
                match result {
                    Ok(status) if status.success() => Ok(Running::finished()),
                    other => Err(UnitError::Exited(describe(other))),
                }
            }
            UnitKind::Service => {
                if let Some(probe) = &self.spec.ready {
                    tokio::select! {
                        result = &mut exit_rx => {
                            return Err(UnitError::Exited(describe(result.unwrap_or_else(|_| Err(lost())))));
                        }
                        () = probe::wait_ready(probe, self.probe_interval) => {}
                    }
                }
                Ok(Running {
                    exited: Box::pin(async move {
                        describe(exit_rx.await.unwrap_or_else(|_| Err(lost())))
                    }),
                })
            }
        }
    }

    async fn stop(&self) -> Result<(), UnitError> {
        let Some(group) = lock(&self.group).take() else {
            return Ok(());
        };
        let result = terminate(group, self.grace).await;
        self.registry.remove(group);
        result
    }
}

fn lost() -> String {
    "exit status lost".to_owned()
}

fn describe(result: ExitResult) -> String {
    match result {
        Ok(status) => format!("process {status}"),
        Err(err) => err,
    }
}

async fn terminate(group: i32, grace: Duration) -> Result<(), UnitError> {
    let Some(pid) = Pid::from_raw(group) else {
        return Ok(());
    };
    if kill_process_group(pid, Signal::TERM).is_err() {
        return Ok(());
    }
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if test_kill_process_group(pid).is_err() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    match kill_process_group(pid, Signal::KILL) {
        Err(err) if err != Errno::SRCH => Err(UnitError::Io(err.into())),
        _ => Ok(()),
    }
}

/// Reads raw bytes so a non-UTF-8 line cannot end the forwarding.
pub(crate) fn forward_lines<R>(reader: Option<R>, sink: LogSink) -> Option<JoinHandle<()>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let reader = reader?;
    Some(tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => sink.line(&String::from_utf8_lossy(&buf)),
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rustix::process::test_kill_process;

    use super::*;
    use crate::units::probe::tests::closed_port;

    fn spec(kind: UnitKind, command: &str) -> ProcessSpec {
        ProcessSpec {
            name: "t".into(),
            kind,
            command: command.into(),
            cwd: std::env::temp_dir(),
            env: BTreeMap::new(),
            ready: None,
            log_format: LogFormat::Plain,
        }
    }

    fn unit(spec: ProcessSpec) -> ProcessUnit {
        ProcessUnit::new(spec, ProcessRegistry::in_memory()).with_grace(Duration::from_secs(1))
    }

    async fn pid_from(path: &Path) -> Pid {
        for _ in 0..100 {
            if let Some(pid) = std::fs::read_to_string(path)
                .ok()
                .and_then(|text| text.trim().parse().ok())
                .and_then(Pid::from_raw)
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("child pid was never written");
    }

    async fn gone(pid: Pid) -> bool {
        for _ in 0..40 {
            if test_kill_process(pid).is_err() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    #[tokio::test]
    async fn oneshot_success_captures_all_output() {
        let store = LogStore::new(100);
        let mut env = BTreeMap::new();
        env.insert("GECO_GREETING".to_owned(), "hello".to_owned());
        let mut spec = spec(UnitKind::Oneshot, "echo $GECO_GREETING; echo oops >&2");
        spec.env = env;

        unit(spec).start(Arc::clone(&store)).await.unwrap();

        let mut messages: Vec<String> = store
            .snapshot(&[], 10)
            .into_iter()
            .map(|r| r.message)
            .collect();
        messages.sort();
        assert_eq!(messages, ["hello", "oops"]);
    }

    #[tokio::test]
    async fn oneshot_failure_reports_the_exit_status() {
        let err = unit(spec(UnitKind::Oneshot, "exit 3"))
            .start(LogStore::new(10))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("exit status: 3"), "{err}");
    }

    #[tokio::test]
    async fn service_without_probe_is_ready_and_reports_its_exit() {
        let running = unit(spec(UnitKind::Service, "sleep 0.2"))
            .start(LogStore::new(10))
            .await
            .unwrap();
        let reason = running.exited.await;
        assert!(reason.contains("exit status: 0"), "{reason}");
    }

    #[tokio::test]
    async fn service_exiting_before_its_probe_succeeds_fails() {
        let mut spec = spec(UnitKind::Service, "exit 1");
        spec.ready = Some(ReadyProbe::Tcp(closed_port().await));
        assert!(matches!(
            unit(spec).start(LogStore::new(10)).await,
            Err(UnitError::Exited(_))
        ));
    }

    #[tokio::test]
    async fn stop_terminates_the_whole_process_group() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("child.pid");
        let unit = unit(spec(
            UnitKind::Service,
            &format!("sleep 30 & echo $! > {}; wait", pid_file.display()),
        ));
        let _running = unit.start(LogStore::new(10)).await.unwrap();
        let grandchild = pid_from(&pid_file).await;

        unit.stop().await.unwrap();

        assert!(
            gone(grandchild).await,
            "the backgrounded sleep survived stop()"
        );
    }

    #[tokio::test]
    async fn registry_tracks_the_running_group() {
        let registry = ProcessRegistry::in_memory();
        let unit = ProcessUnit::new(spec(UnitKind::Service, "sleep 30"), registry.clone())
            .with_grace(Duration::from_secs(1));

        let _running = unit.start(LogStore::new(10)).await.unwrap();
        assert_eq!(registry.groups().len(), 1);

        unit.stop().await.unwrap();
        assert!(registry.groups().is_empty());
    }

    #[tokio::test]
    async fn a_missing_working_directory_is_a_spawn_error() {
        let mut spec = spec(UnitKind::Service, "true");
        spec.cwd = PathBuf::from("/nonexistent/geco");
        assert!(matches!(
            unit(spec).start(LogStore::new(10)).await,
            Err(UnitError::Spawn { .. })
        ));
    }
}
