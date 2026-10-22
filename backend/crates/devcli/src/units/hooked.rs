use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tokio::process::Command;

use super::{Origin, Running, Unit, UnitError, UnitKind, process::output_in_own_group};
use crate::{
    config::LogFormat,
    logs::{LogSink, LogStore},
};

#[async_trait]
pub trait Verify: Send + Sync {
    async fn verify(&self) -> Result<(), UnitError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevServicesFile {
    pub template: PathBuf,
    pub target: PathBuf,
    pub app_host: String,
    pub entrypoint: String,
}

impl DevServicesFile {
    /// The same substitution `just run-dev` performs with sed.
    pub fn render(&self, template: &str) -> String {
        template
            .replace("${APP_HOST}", &self.app_host)
            .replace("${TRAEFIK_ENTRYPOINT}", &self.entrypoint)
    }

    fn write(&self) -> io::Result<()> {
        let template = fs::read_to_string(&self.template)?;
        fs::write(&self.target, self.render(&template))
    }

    fn remove(&self) -> io::Result<()> {
        match fs::remove_file(&self.target) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

pub struct HookedUnit {
    inner: Arc<dyn Unit>,
    root: PathBuf,
    pre_start: Vec<String>,
    dev_services: Option<DevServicesFile>,
    verify: Option<Arc<dyn Verify>>,
}

impl HookedUnit {
    pub fn new(inner: Arc<dyn Unit>, root: PathBuf) -> Self {
        Self {
            inner,
            root,
            pre_start: Vec::new(),
            dev_services: None,
            verify: None,
        }
    }

    pub fn pre_start(mut self, commands: Vec<String>) -> Self {
        self.pre_start = commands;
        self
    }

    pub fn dev_services(mut self, file: DevServicesFile) -> Self {
        self.dev_services = Some(file);
        self
    }

    pub fn verify(mut self, verify: Arc<dyn Verify>) -> Self {
        self.verify = Some(verify);
        self
    }
}

#[async_trait]
impl Unit for HookedUnit {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn kind(&self) -> UnitKind {
        self.inner.kind()
    }

    fn origin(&self) -> Origin {
        self.inner.origin()
    }

    async fn start(&self, logs: Arc<LogStore>) -> Result<Running, UnitError> {
        let sink = LogSink::new(Arc::clone(&logs), self.inner.name(), LogFormat::Plain);
        for command in &self.pre_start {
            run_hook(command, &self.root, &sink).await?;
        }
        if let Some(file) = &self.dev_services {
            file.write().map_err(|err| {
                UnitError::Hook(format!("cannot write {}: {err}", file.target.display()))
            })?;
        }
        let running = self.inner.start(logs).await?;
        if let Some(verify) = &self.verify {
            verify.verify().await?;
        }
        Ok(running)
    }

    async fn stop(&self) -> Result<(), UnitError> {
        let stopped = self.inner.stop().await;
        if let Some(file) = &self.dev_services {
            file.remove().map_err(|err| {
                UnitError::Hook(format!("cannot remove {}: {err}", file.target.display()))
            })?;
        }
        stopped
    }
}

async fn run_hook(command: &str, root: &Path, sink: &LogSink) -> Result<(), UnitError> {
    let mut shell = Command::new("sh");
    shell.arg("-c").arg(command).current_dir(root);
    let output = output_in_own_group(shell)
        .await
        .map_err(|source| UnitError::Spawn {
            command: command.to_owned(),
            source,
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stdout.lines().chain(stderr.lines()) {
        sink.line(line);
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(UnitError::Hook(format!(
            "`{command}` failed with {}",
            output.status
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logs::Filter,
        units::{
            fake::{FakeUnit, Journal},
            process::tests::{gone, pid_from},
        },
    };

    fn dev_services(dir: &Path) -> DevServicesFile {
        fs::write(
            dir.join("dev-services.yaml.tmpl"),
            "rule: Host(`${APP_HOST}`)\nentryPoints: [${TRAEFIK_ENTRYPOINT}]\n",
        )
        .unwrap();
        DevServicesFile {
            template: dir.join("dev-services.yaml.tmpl"),
            target: dir.join("dev-services.yaml"),
            app_host: "geco.test".into(),
            entrypoint: "websecure".into(),
        }
    }

    struct Failing;

    #[async_trait]
    impl Verify for Failing {
        async fn verify(&self) -> Result<(), UnitError> {
            Err(UnitError::Verify("2 migrations missing".into()))
        }
    }

    #[tokio::test]
    async fn runs_hooks_around_the_inner_unit() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::default();
        let file = dev_services(dir.path());
        let target = file.target.clone();
        let unit = HookedUnit::new(
            Arc::new(FakeUnit::service("proxy", &journal)),
            dir.path().into(),
        )
        .pre_start(vec!["echo hook-ran".into()])
        .dev_services(file);
        let store = LogStore::new(10);

        unit.start(Arc::clone(&store)).await.unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "rule: Host(`geco.test`)\nentryPoints: [websecure]\n"
        );
        let all = Filter::default();
        let snapshot = store.snapshot(&["proxy".into()], &all, 10, chrono::Utc::now());
        assert_eq!(snapshot[0].message, "hook-ran");
        assert_eq!(journal.entries(), ["start proxy"]);

        unit.stop().await.unwrap();
        assert!(!target.exists());
        assert_eq!(journal.entries(), ["start proxy", "stop proxy"]);
    }

    #[tokio::test]
    async fn a_failing_hook_skips_the_inner_unit() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::default();
        let unit = HookedUnit::new(
            Arc::new(FakeUnit::service("proxy", &journal)),
            dir.path().into(),
        )
        .pre_start(vec!["exit 1".into()]);

        assert!(matches!(
            unit.start(LogStore::new(10)).await,
            Err(UnitError::Hook(_))
        ));
        assert!(journal.entries().is_empty());
    }

    #[tokio::test]
    async fn dropping_a_running_hook_kills_it() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("sleep.pid");
        let command = format!("sleep 30 & echo $! > {}; wait", pid_file.display());
        let sink = LogSink::new(LogStore::new(10), "proxy", LogFormat::Plain);
        let mut hook = Box::pin(run_hook(&command, dir.path(), &sink));

        let sleep = tokio::select! {
            _ = &mut hook => panic!("the hook finished on its own"),
            pid = pid_from(&pid_file) => pid,
        };
        drop(hook);

        assert!(gone(sleep).await, "the hook's sleep survived the drop");
    }

    #[tokio::test]
    async fn a_failed_verification_fails_the_start() {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::default();
        let unit = HookedUnit::new(
            Arc::new(FakeUnit::oneshot("migrations", &journal)),
            dir.path().into(),
        )
        .verify(Arc::new(Failing));

        let err = unit.start(LogStore::new(10)).await.unwrap_err();
        assert_eq!(err.to_string(), "2 migrations missing");
    }
}
