use std::process::ExitCode;

use anyhow::Context;

use super::output;
use crate::{
    config::GecoConfig,
    host::confirm,
    ipc::{Call, Client, IpcError},
    paths::Paths,
    tasks::{Catalog, JustRecipes, RecipeSource},
};

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
    let interrupted = tokio::signal::ctrl_c();
    tokio::pin!(interrupted);
    let result = tokio::select! {
        result = client.run(recipe, args, destructive, |record| output::write_log(&mut out, &record)) => result,
        _ = &mut interrupted => {
            let _ = client.call(Call::Stop { unit: task }).await;
            anstream::eprintln!("stopped");
            return Ok(ExitCode::from(130));
        }
    };
    match result {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(IpcError::NotRunning) => in_foreground(paths, recipe, args),
        Err(IpcError::Remote(message)) => {
            anstream::eprintln!("error: {message}");
            Ok(ExitCode::FAILURE)
        }
        Err(err) => Err(err.into()),
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
