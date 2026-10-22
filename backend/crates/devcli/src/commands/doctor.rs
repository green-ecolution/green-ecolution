use std::{collections::BTreeSet, io::Write, process::ExitCode, sync::Arc};

use anyhow::{Context, bail};

use super::output;
use crate::{
    config::JustVars,
    doctor::{self, Env, Report},
    host::confirm,
    ipc::{Call, Client, IpcError, Reply},
    paths::Paths,
    units::compose::ComposeCli,
};

pub async fn doctor(
    paths: &Paths,
    client: &Client,
    fix: bool,
    deep: bool,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let mut report = diagnose(paths, client, deep).await?;
    if fix && apply_fixes(paths, &report)? {
        report = diagnose(paths, client, deep).await?;
    }
    let mut out = anstream::stdout();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&report)?)?;
    } else {
        output::write_report(&mut out, &report)?;
    }
    Ok(if report.has_errors() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

async fn diagnose(paths: &Paths, client: &Client, deep: bool) -> anyhow::Result<Report> {
    match client.call(Call::Doctor { deep }).await {
        Ok(Reply::Report(report)) => Ok(report),
        Ok(other) => bail!("unexpected reply from the supervisor: {other:?}"),
        Err(IpcError::NotRunning) => {
            let env = Env {
                root: paths.root.clone(),
                compose: ComposeCli::new(paths.root.clone()),
                vars: Arc::new(JustVars::new(paths.root.clone())),
                units: Vec::new(),
            };
            Ok(doctor::run(&doctor::checks::all(), &env, deep).await)
        }
        Err(err) => Err(err.into()),
    }
}

/// Runs in the foreground so the recipe's output and prompts reach this terminal; each
/// command runs once even when several findings share it.
fn apply_fixes(paths: &Paths, report: &Report) -> anyhow::Result<bool> {
    let mut ran = BTreeSet::new();
    for finding in &report.findings {
        let Some(fix) = &finding.fix else {
            continue;
        };
        if ran.contains(&fix.command) {
            continue;
        }
        anstream::eprintln!("→ {} ({})", fix.command, finding.title);
        if fix.destructive
            && !confirm(&format!("`{}` is destructive. Run it? [y/N] ", fix.command))?
        {
            continue;
        }
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&fix.command)
            .current_dir(paths.root.join(&fix.cwd))
            .status()
            .with_context(|| format!("cannot run `{}`", fix.command))?;
        if !status.success() {
            anstream::eprintln!("`{}` failed ({status})", fix.command);
        }
        ran.insert(fix.command.clone());
    }
    Ok(!ran.is_empty())
}
