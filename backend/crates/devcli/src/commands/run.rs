use std::process::ExitCode;

use anyhow::Context;
use tokio::sync::mpsc;

use super::output;
use crate::{
    config::GecoConfig,
    host::confirm,
    ipc::{Client, IpcError, RunEnd},
    paths::Paths,
    tasks::{Catalog, JustRecipes, RecipeSource},
};

const INTERRUPTED: u8 = 130;

pub async fn run(
    paths: &Paths,
    client: &Client,
    recipe: &str,
    args: &[String],
    yes: bool,
) -> anyhow::Result<ExitCode> {
    let config = GecoConfig::load(&paths.config())?;
    let catalog = Catalog::parse(&JustRecipes::new(paths.root.clone()).dump()?, &config.tasks)?;
    let known = catalog.get(recipe);
    let destructive = known.is_some_and(|recipe| recipe.destructive);
    if destructive && !yes && !confirm(&format!("`just {recipe}` is destructive. Run it? [y/N] "))?
    {
        return Ok(ExitCode::FAILURE);
    }
    let task = known
        .map_or(recipe, |recipe| recipe.name.as_str())
        .to_owned();
    let mut out = anstream::stdout();
    let (interrupt, mut interrupts) = mpsc::unbounded_channel();
    let signals = tokio::spawn(async move {
        while tokio::signal::ctrl_c().await.is_ok() && interrupt.send(()).is_ok() {}
    });
    let result = client
        .run_interruptible(
            recipe,
            args,
            destructive,
            &task,
            &mut interrupts,
            |record| output::write_log(&mut out, &record),
        )
        .await;
    signals.abort();
    match result {
        Ok(RunEnd::Done) => Ok(ExitCode::SUCCESS),
        Ok(end) => {
            anstream::eprintln!("{}", interrupted(&task, &end));
            Ok(ExitCode::from(INTERRUPTED))
        }
        Err(IpcError::NotRunning) => in_foreground(paths, recipe, args),
        Err(IpcError::Remote(message)) => {
            anstream::eprintln!("error: {message}");
            Ok(ExitCode::FAILURE)
        }
        Err(err) => Err(err.into()),
    }
}

fn interrupted(task: &str, end: &RunEnd) -> String {
    match end {
        RunEnd::Stopped => "stopped".to_owned(),
        RunEnd::Done | RunEnd::Ended(Ok(())) => {
            format!("`{task}` finished before it could be stopped")
        }
        RunEnd::Ended(Err(message)) => format!("error: {message}"),
        RunEnd::Abandoned => {
            format!("no longer waiting for `{task}` to stop; see `geco status`")
        }
    }
}

/// Without a supervisor the recipe gets this terminal, prompts included.
fn in_foreground(paths: &Paths, recipe: &str, args: &[String]) -> anyhow::Result<ExitCode> {
    let status = std::process::Command::new("just")
        .arg(recipe)
        .args(args)
        .current_dir(&paths.root)
        .status()
        .context("cannot run just")?;
    Ok(exit_code(status))
}

fn exit_code(status: std::process::ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    #[test]
    fn an_interrupted_run_says_stopped_only_when_the_stop_took_effect() {
        assert_eq!(interrupted("lint", &RunEnd::Stopped), "stopped");
        assert_eq!(
            interrupted("lint", &RunEnd::Ended(Ok(()))),
            "`lint` finished before it could be stopped"
        );
        assert_eq!(
            interrupted("lint", &RunEnd::Ended(Err("`just lint` failed".into()))),
            "error: `just lint` failed"
        );
        assert!(interrupted("lint", &RunEnd::Abandoned).contains("geco status"));
    }

    #[test]
    fn exit_codes_of_just_are_passed_through() {
        assert_eq!(
            exit_code(std::process::ExitStatus::from_raw(0)),
            ExitCode::SUCCESS
        );
        assert_eq!(
            exit_code(std::process::ExitStatus::from_raw(3 << 8)),
            ExitCode::from(3)
        );
        // Killed by a signal: no exit code.
        assert_eq!(
            exit_code(std::process::ExitStatus::from_raw(9)),
            ExitCode::FAILURE
        );
    }
}
