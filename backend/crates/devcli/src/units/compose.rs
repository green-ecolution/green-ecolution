use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use bollard::{
    Docker,
    container::LogOutput,
    models::{ContainerState, ContainerStateStatusEnum, HealthStatusEnum},
    query_parameters::{InspectContainerOptions, ListContainersOptionsBuilder, LogsOptionsBuilder},
};
use futures::StreamExt;
use serde::Deserialize;
use tokio::{process::Command, task::JoinHandle};

use super::{Origin, Running, Unit, UnitError, UnitKind, process::output_in_own_group};
use crate::{
    config::LogFormat,
    logs::{LogSink, LogStore},
    sync::lock,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeProject {
    pub name: String,
    pub services: BTreeMap<String, ComposeService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeService {
    pub depends_on: Vec<String>,
    pub oneshot: bool,
}

// Only the fields we need are declared, so environment values are never deserialized.
#[derive(Deserialize)]
struct RawConfig {
    name: String,
    #[serde(default)]
    services: BTreeMap<String, RawService>,
}

#[derive(Deserialize)]
struct RawService {
    #[serde(default)]
    depends_on: BTreeMap<String, RawDependency>,
}

#[derive(Deserialize)]
struct RawDependency {
    condition: Option<String>,
}

/// `docker compose config` only lists services of active compose profiles, so the `app` and
/// `plugins` services never collide with local units of the same name.
pub fn parse_config(json: &str) -> Result<ComposeProject, serde_json::Error> {
    let raw: RawConfig = serde_json::from_str(json)?;
    let awaited_to_finish: BTreeSet<&str> = raw
        .services
        .values()
        .flat_map(|service| &service.depends_on)
        .filter(|(_, dep)| dep.condition.as_deref() == Some("service_completed_successfully"))
        .map(|(name, _)| name.as_str())
        .collect();
    let services = raw
        .services
        .iter()
        .map(|(name, service)| {
            let service = ComposeService {
                depends_on: service.depends_on.keys().cloned().collect(),
                oneshot: awaited_to_finish.contains(name.as_str()),
            };
            (name.clone(), service)
        })
        .collect();
    Ok(ComposeProject {
        name: raw.name,
        services,
    })
}

#[derive(Debug, Clone)]
pub struct ComposeCli {
    root: PathBuf,
}

impl ComposeCli {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Goes through `just _compose` so the environment compose needs stays defined in the
    /// justfile alone. The output can contain secrets and is never logged.
    pub async fn run(&self, args: &[&str]) -> Result<String, UnitError> {
        let mut command = Command::new("just");
        command.arg("_compose").args(args).current_dir(&self.root);
        let output = output_in_own_group(command)
            .await
            .map_err(|source| UnitError::Spawn {
                command: format!("just _compose {}", args.join(" ")),
                source,
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        Err(UnitError::Docker(format!(
            "`docker compose {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }

    pub async fn project(&self) -> Result<ComposeProject, UnitError> {
        let json = self.run(&["config", "--format", "json"]).await?;
        parse_config(&json)
            .map_err(|err| UnitError::Docker(format!("cannot read the compose config: {err}")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Readiness {
    Pending,
    Ready,
    Done,
    Failed(String),
}

pub fn readiness(state: &ContainerState, kind: UnitKind) -> Readiness {
    use ContainerStateStatusEnum as Status;
    match state.status {
        Some(Status::EXITED | Status::DEAD) => match (state.exit_code, kind) {
            (Some(0), UnitKind::Oneshot) => Readiness::Done,
            (code, _) => Readiness::Failed(format!(
                "container exited with code {}",
                code.map_or_else(|| "unknown".to_owned(), |code| code.to_string())
            )),
        },
        Some(Status::RUNNING) => {
            let health = state.health.as_ref().and_then(|health| health.status);
            match (health, kind) {
                (Some(HealthStatusEnum::HEALTHY), _) => Readiness::Ready,
                (Some(HealthStatusEnum::STARTING | HealthStatusEnum::UNHEALTHY), _) => {
                    Readiness::Pending
                }
                (_, UnitKind::Oneshot) => Readiness::Pending,
                (_, UnitKind::Service) => Readiness::Ready,
            }
        }
        _ => Readiness::Pending,
    }
}

pub struct ComposeUnit {
    service: String,
    project: String,
    kind: UnitKind,
    docker: Docker,
    cli: ComposeCli,
    poll: Duration,
    log_task: Mutex<Option<JoinHandle<()>>>,
}

impl ComposeUnit {
    pub fn new(
        service: String,
        project: String,
        kind: UnitKind,
        docker: Docker,
        cli: ComposeCli,
    ) -> Self {
        Self {
            service,
            project,
            kind,
            docker,
            cli,
            poll: Duration::from_secs(1),
            log_task: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Unit for ComposeUnit {
    fn name(&self) -> &str {
        &self.service
    }

    fn kind(&self) -> UnitKind {
        self.kind
    }

    fn origin(&self) -> Origin {
        Origin::Compose
    }

    async fn start(&self, logs: Arc<LogStore>) -> Result<Running, UnitError> {
        // One second of slack so lines emitted right at startup are not cut off by rounding.
        let since = i32::try_from(chrono::Utc::now().timestamp() - 1).unwrap_or(0);
        // The supervisor already orders the graph, so compose must not start dependencies itself.
        self.cli
            .run(&["up", "--detach", "--no-deps", self.service.as_str()])
            .await?;
        let id = find_container(&self.docker, &self.project, &self.service).await?;
        let sink = LogSink::new(logs, &self.service, LogFormat::Plain);
        let task = tokio::spawn(follow_logs(self.docker.clone(), id.clone(), sink, since));
        if let Some(previous) = lock(&self.log_task).replace(task) {
            previous.abort();
        }
        loop {
            match readiness(&inspect_state(&self.docker, &id).await?, self.kind) {
                Readiness::Pending => tokio::time::sleep(self.poll).await,
                Readiness::Ready => {
                    return Ok(Running {
                        exited: Box::pin(watch_exit(self.docker.clone(), id, self.poll)),
                    });
                }
                Readiness::Done => return Ok(Running::finished()),
                Readiness::Failed(reason) => return Err(UnitError::Docker(reason)),
            }
        }
    }

    async fn stop(&self) -> Result<(), UnitError> {
        if let Some(task) = lock(&self.log_task).take() {
            task.abort();
        }
        self.cli
            .run(&["stop", self.service.as_str()])
            .await
            .map(drop)
    }
}

fn docker_error(err: bollard::errors::Error) -> UnitError {
    UnitError::Docker(err.to_string())
}

pub async fn find_container(
    docker: &Docker,
    project: &str,
    service: &str,
) -> Result<String, UnitError> {
    let filters = HashMap::from([(
        "label",
        vec![
            format!("com.docker.compose.project={project}"),
            format!("com.docker.compose.service={service}"),
        ],
    )]);
    let options = ListContainersOptionsBuilder::new()
        .all(true)
        .filters(&filters)
        .build();
    docker
        .list_containers(Some(options))
        .await
        .map_err(docker_error)?
        .into_iter()
        .find_map(|container| container.id)
        .ok_or_else(|| UnitError::Docker(format!("no container for compose service `{service}`")))
}

/// Service name to Docker's human-readable status, for `geco status` without a supervisor.
pub async fn service_overview(
    docker: &Docker,
    project: &str,
) -> Result<BTreeMap<String, String>, UnitError> {
    let filters = HashMap::from([(
        "label",
        vec![format!("com.docker.compose.project={project}")],
    )]);
    let options = ListContainersOptionsBuilder::new()
        .all(true)
        .filters(&filters)
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(docker_error)?;
    Ok(containers
        .into_iter()
        .filter_map(|container| {
            let service = container.labels?.get("com.docker.compose.service")?.clone();
            Some((service, container.status.unwrap_or_default()))
        })
        .collect())
}

async fn inspect_state(docker: &Docker, id: &str) -> Result<ContainerState, UnitError> {
    docker
        .inspect_container(id, None::<InspectContainerOptions>)
        .await
        .map_err(docker_error)?
        .state
        .ok_or_else(|| UnitError::Docker("container reports no state".to_owned()))
}

async fn watch_exit(docker: Docker, id: String, poll: Duration) -> String {
    loop {
        tokio::time::sleep(poll).await;
        match inspect_state(&docker, &id).await {
            Ok(state) if state.running == Some(true) => {}
            Ok(state) => {
                return format!(
                    "container stopped (exit code {})",
                    state.exit_code.unwrap_or(-1)
                );
            }
            Err(err) => return err.to_string(),
        }
    }
}

/// Splits a byte stream into lines, decoding per complete line so a multibyte character split
/// across frames survives.
#[derive(Default)]
struct LineBuffer {
    pending: Vec<u8>,
}

impl LineBuffer {
    fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(bytes);
        let mut lines = Vec::new();
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            lines.push(String::from_utf8_lossy(&self.pending[..end]).into_owned());
            self.pending.drain(..=end);
        }
        lines
    }

    fn finish(self) -> Option<String> {
        (!self.pending.is_empty()).then(|| String::from_utf8_lossy(&self.pending).into_owned())
    }
}

async fn follow_logs(docker: Docker, id: String, sink: LogSink, since: i32) {
    let options = LogsOptionsBuilder::new()
        .follow(true)
        .stdout(true)
        .stderr(true)
        .since(since)
        .build();
    let mut stream = docker.logs(&id, Some(options));
    let mut stdout = LineBuffer::default();
    let mut stderr = LineBuffer::default();
    while let Some(frame) = stream.next().await {
        let frame = match frame {
            Ok(frame) => frame,
            Err(err) => {
                sink.line(&format!("log stream ended: {err}"));
                break;
            }
        };
        let buffer = match frame {
            LogOutput::StdErr { .. } => &mut stderr,
            _ => &mut stdout,
        };
        for line in buffer.push(&frame.into_bytes()) {
            sink.line(&line);
        }
    }
    for line in [stdout.finish(), stderr.finish()].into_iter().flatten() {
        sink.line(&line);
    }
}

#[cfg(test)]
mod tests {
    use bollard::models::Health;

    use super::*;

    #[test]
    fn parses_services_dependencies_and_oneshots() {
        let json = r#"{
          "name": "green-ecolution",
          "services": {
            "db": {"image": "postgis", "environment": {"POSTGRES_PASSWORD": "secret"}},
            "keycloak-init-db": {"depends_on": {"db": {"condition": "service_healthy", "required": true}}},
            "keycloak": {"depends_on": {"keycloak-init-db": {"condition": "service_completed_successfully", "required": true}}}
          }
        }"#;

        let project = parse_config(json).unwrap();

        assert_eq!(project.name, "green-ecolution");
        assert_eq!(
            project.services["keycloak"].depends_on,
            ["keycloak-init-db"]
        );
        assert!(project.services["keycloak-init-db"].oneshot);
        assert!(!project.services["db"].oneshot);
        assert!(project.services["db"].depends_on.is_empty());
    }

    fn state(
        status: ContainerStateStatusEnum,
        exit_code: Option<i64>,
        health: Option<HealthStatusEnum>,
    ) -> ContainerState {
        ContainerState {
            status: Some(status),
            exit_code,
            health: health.map(|status| Health {
                status: Some(status),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn line_buffer_decodes_multibyte_characters_split_across_pushes() {
        let bytes = "grün\n".as_bytes();
        let mut buffer = LineBuffer::default();

        assert!(buffer.push(&bytes[..3]).is_empty());
        assert_eq!(buffer.push(&bytes[3..]), ["grün"]);
    }

    #[test]
    fn line_buffer_splits_several_lines_in_one_chunk() {
        let mut buffer = LineBuffer::default();

        assert_eq!(buffer.push(b"one\ntwo\nthr"), ["one", "two"]);
        assert_eq!(buffer.push(b"ee\n"), ["three"]);
        assert_eq!(buffer.finish(), None);
    }

    #[test]
    fn line_buffer_returns_trailing_partial_line_on_finish() {
        let mut buffer = LineBuffer::default();
        buffer.push(b"done\npartial");

        assert_eq!(buffer.finish().as_deref(), Some("partial"));
    }

    #[test]
    fn maps_container_state_to_readiness() {
        use ContainerStateStatusEnum::{CREATED, EXITED, RUNNING};
        use HealthStatusEnum::{HEALTHY, STARTING, UNHEALTHY};
        use UnitKind::{Oneshot, Service};

        let cases = [
            (
                state(RUNNING, None, Some(HEALTHY)),
                Service,
                Readiness::Ready,
            ),
            (
                state(RUNNING, None, Some(STARTING)),
                Service,
                Readiness::Pending,
            ),
            (
                state(RUNNING, None, Some(UNHEALTHY)),
                Service,
                Readiness::Pending,
            ),
            (state(RUNNING, None, None), Service, Readiness::Ready),
            (state(RUNNING, None, None), Oneshot, Readiness::Pending),
            (state(EXITED, Some(0), None), Oneshot, Readiness::Done),
            (
                state(EXITED, Some(0), None),
                Service,
                Readiness::Failed("container exited with code 0".into()),
            ),
            (
                state(EXITED, Some(1), None),
                Oneshot,
                Readiness::Failed("container exited with code 1".into()),
            ),
            (state(CREATED, None, None), Service, Readiness::Pending),
        ];

        for (container, kind, expected) in cases {
            assert_eq!(
                readiness(&container, kind),
                expected,
                "{container:?} as {kind:?}"
            );
        }
    }
}
