pub mod catalog;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

#[cfg(test)]
pub use catalog::StaticRecipes;
pub use catalog::{Catalog, CatalogError, JustRecipes, Param, ParamKind, Recipe, RecipeSource};

use crate::{
    config::{ConfigError, GecoConfig, LogFormat},
    logs::{LogSink, LogStore},
    paths::Paths,
    state::ProcessRegistry,
    supervisor::{Supervisor, SupervisorError, TaskOutcome},
    units::{
        Origin, Running, Unit, UnitError, UnitKind,
        process::{ProcessSpec, ProcessUnit},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSpec {
    pub name: String,
    pub command: String,
    pub cwd: PathBuf,
    pub destructive: bool,
}

/// Ends the message of [`TaskError::Stopped`]; `geco run` recognises a stopped run by it.
pub const STOPPED: &str = "was stopped before it finished";

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("unknown recipe `{0}`")]
    UnknownRecipe(String),
    #[error("`{0}` is destructive and needs confirmation")]
    NeedsConfirmation(String),
    #[error("`{command}` failed: {reason}")]
    Failed { command: String, reason: String },
    #[error("`{0}` {STOPPED}")]
    Stopped(String),
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
}

pub struct Tasks {
    paths: Paths,
    registry: ProcessRegistry,
    supervisor: Arc<Supervisor>,
    recipes: Arc<dyn RecipeSource>,
    program: String,
}

impl Tasks {
    pub fn new(root: PathBuf, registry: ProcessRegistry, supervisor: Arc<Supervisor>) -> Self {
        Self {
            recipes: Arc::new(JustRecipes::new(root.clone())),
            paths: Paths::new(root),
            registry,
            supervisor,
            program: "just".to_owned(),
        }
    }

    #[cfg(test)]
    pub fn with_recipes(mut self, recipes: Arc<dyn RecipeSource>) -> Self {
        self.recipes = recipes;
        self
    }

    /// Lets tests run `echo <recipe> …` instead of needing `just` and a justfile.
    #[cfg(test)]
    pub fn with_program(mut self, program: &str) -> Self {
        program.clone_into(&mut self.program);
        self
    }

    /// Reads `geco.toml` again each time, so an edited destructive list applies at once.
    pub fn catalog(&self) -> Result<Catalog, TaskError> {
        let config = GecoConfig::load(&self.paths.config())?;
        Ok(Catalog::parse(&self.recipes.dump()?, &config.tasks)?)
    }

    pub fn recipe(&self, name: &str, args: &[String]) -> Result<TaskSpec, TaskError> {
        let catalog = self.catalog()?;
        let recipe = catalog
            .get(name)
            .ok_or_else(|| TaskError::UnknownRecipe(name.to_owned()))?;
        Ok(TaskSpec {
            name: recipe.name.clone(),
            command: recipe.command(&self.program, args),
            cwd: self.paths.root.clone(),
            destructive: recipe.destructive,
        })
    }

    /// `[tasks] destructive` can only add to what the caller (a doctor check) decided.
    pub fn command(
        &self,
        name: &str,
        command: &str,
        cwd: &Path,
        destructive: bool,
    ) -> Result<TaskSpec, TaskError> {
        let config = GecoConfig::load(&self.paths.config())?;
        Ok(TaskSpec {
            name: name.to_owned(),
            command: command.to_owned(),
            cwd: self.paths.root.join(cwd),
            destructive: destructive || config.tasks.is_destructive(command),
        })
    }

    pub async fn run(&self, spec: TaskSpec, confirmed: bool) -> Result<(), TaskError> {
        if spec.destructive && !confirmed {
            return Err(TaskError::NeedsConfirmation(spec.command));
        }
        let unit = TaskUnit {
            inner: ProcessUnit::new(
                ProcessSpec {
                    name: spec.name,
                    kind: UnitKind::Oneshot,
                    command: spec.command.clone(),
                    cwd: spec.cwd,
                    env: BTreeMap::new(),
                    ready: None,
                    log_format: LogFormat::Plain,
                },
                self.registry.clone(),
            ),
            command: spec.command.clone(),
        };
        match self.supervisor.run_task(Arc::new(unit)).await? {
            TaskOutcome::Done => Ok(()),
            TaskOutcome::Failed(reason) => Err(TaskError::Failed {
                command: spec.command,
                reason,
            }),
            TaskOutcome::Stopped => Err(TaskError::Stopped(spec.command)),
        }
    }
}

/// Logs its command first, as a shell would; fix commands carry no secrets by contract.
struct TaskUnit {
    inner: ProcessUnit,
    command: String,
}

#[async_trait]
impl Unit for TaskUnit {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn kind(&self) -> UnitKind {
        UnitKind::Oneshot
    }

    fn origin(&self) -> Origin {
        Origin::Task
    }

    fn definition(&self) -> u64 {
        self.inner.definition()
    }

    async fn start(&self, logs: Arc<LogStore>) -> Result<Running, UnitError> {
        LogSink::new(Arc::clone(&logs), self.inner.name(), LogFormat::Plain)
            .line(&format!("$ {}", self.command));
        self.inner.start(logs).await
    }

    async fn stop(&self) -> Result<(), UnitError> {
        self.inner.stop().await
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Arc};

    use chrono::Utc;

    use super::*;
    use crate::{
        graph::Graph,
        logs::{Filter, LogStore},
        supervisor::{Supervisor, UnitState},
        units::{
            Origin, Unit,
            fake::{FakeUnit, Journal},
        },
    };

    struct Fixture {
        _dir: tempfile::TempDir,
        supervisor: Arc<Supervisor>,
        tasks: Tasks,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("geco.toml"),
            "[tasks]\ndestructive = [\"migrate-reset\"]\n",
        )
        .unwrap();
        let units: Vec<Arc<dyn Unit>> =
            vec![Arc::new(FakeUnit::service("db", &Journal::default()))];
        let graph = Graph::new([("db".to_owned(), Vec::<String>::new())]).unwrap();
        let supervisor = Supervisor::new(units, graph, LogStore::new(100));
        let tasks = Tasks::new(
            dir.path().to_path_buf(),
            ProcessRegistry::in_memory(),
            Arc::clone(&supervisor),
        )
        .with_recipes(Arc::new(StaticRecipes(catalog::tests::DUMP.to_owned())))
        .with_program("echo");
        Fixture {
            _dir: dir,
            supervisor,
            tasks,
        }
    }

    fn logs(f: &Fixture, unit: &str) -> Vec<String> {
        f.supervisor
            .logs()
            .snapshot(&[unit.to_owned()], &Filter::default(), 100, Utc::now())
            .into_iter()
            .map(|record| record.message)
            .collect()
    }

    #[tokio::test]
    async fn a_recipe_runs_as_a_task_and_logs_its_command() {
        let f = fixture();
        let spec = f.tasks.recipe("migrate-new", &["add foo".into()]).unwrap();
        assert_eq!(spec.name, "migrate-new");
        assert_eq!(spec.command, "echo migrate-new 'add foo'");

        f.tasks.run(spec, false).await.unwrap();

        assert_eq!(
            logs(&f, "migrate-new"),
            ["$ echo migrate-new 'add foo'", "migrate-new add foo"]
        );
        assert!(
            f.supervisor
                .status()
                .iter()
                .all(|s| s.name != "migrate-new")
        );
    }

    #[tokio::test]
    async fn aliases_run_their_recipe() {
        let f = fixture();
        assert_eq!(f.tasks.recipe("fe-dev", &[]).unwrap().name, "frontend-dev");
    }

    #[tokio::test]
    async fn destructive_recipes_need_confirmation() {
        let f = fixture();
        let spec = f.tasks.recipe("migrate-reset", &[]).unwrap();
        assert!(spec.destructive);
        assert!(matches!(
            f.tasks.run(spec.clone(), false).await,
            Err(TaskError::NeedsConfirmation(command)) if command == "echo migrate-reset"
        ));
        f.tasks.run(spec, true).await.unwrap();
    }

    #[tokio::test]
    async fn unknown_recipes_are_refused() {
        let f = fixture();
        assert!(matches!(
            f.tasks.recipe("nope", &[]),
            Err(TaskError::UnknownRecipe(name)) if name == "nope"
        ));
    }

    #[tokio::test]
    async fn a_failing_command_is_reported_and_stays_listed() {
        let f = fixture();
        let spec = f
            .tasks
            .command("broken", "exit 4", Path::new(""), false)
            .unwrap();

        let result = f.tasks.run(spec, false).await;

        assert!(
            matches!(&result, Err(TaskError::Failed { reason, .. }) if reason.contains("exit status: 4")),
            "{result:?}"
        );
        let listed = f
            .supervisor
            .status()
            .into_iter()
            .find(|s| s.name == "broken")
            .unwrap();
        assert_eq!(listed.origin, Origin::Task);
        assert!(matches!(listed.state, UnitState::Failed { .. }));
    }

    #[test]
    fn commands_running_a_listed_recipe_count_as_destructive() {
        let f = fixture();
        assert!(
            f.tasks
                .command("fix", "just migrate-reset", Path::new(""), false)
                .unwrap()
                .destructive
        );
        assert!(
            !f.tasks
                .command("fix", "just migrate-up", Path::new(""), false)
                .unwrap()
                .destructive
        );
        assert!(
            f.tasks
                .command("fix", "pnpm build", Path::new(""), true)
                .unwrap()
                .destructive
        );
    }
}
