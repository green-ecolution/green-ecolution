mod fallback;
mod output;

use std::{
    io::Write,
    os::unix::process::CommandExt,
    process::{ExitCode, Stdio},
    time::Duration,
};

use anyhow::{Context, bail};
use chrono::Utc;

use crate::{
    cli::{Cli, Command},
    host::{Host, OrphanPolicy, confirm, orphan_prompt},
    ipc::{Call, Client, IpcError, Reply},
    paths::Paths,
    state::{InstanceLock, orphaned_groups, stop_orphans},
    tui,
};

pub async fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    let cwd = std::env::current_dir()?;
    let paths = Paths::discover(&cwd).context("no geco.toml in this directory or any parent")?;
    let client = Client::new(paths.socket());
    let profile = cli.profile.as_deref();
    match cli.command {
        None => tui::run(&paths, profile).await,
        Some(Command::Up { detach: false }) => up(&paths, profile).await,
        Some(Command::Up { detach: true }) => up_detached(&paths, &client, profile).await,
        Some(Command::Supervise) => supervise(&paths, profile).await,
        Some(Command::Down) => down(&paths, &client).await,
        Some(Command::Status { json }) => status(&paths, &client, json).await,
        Some(Command::Logs {
            units,
            follow,
            json,
            tail,
        }) => logs(&paths, &client, &units, tail, follow, json).await,
        Some(Command::Restart { unit, cascade }) => {
            simple(&client, Call::Restart { unit, cascade }).await
        }
        Some(Command::Start { unit }) => simple(&client, Call::Start { unit }).await,
        Some(Command::Stop { unit }) => simple(&client, Call::Stop { unit }).await,
    }
}

async fn up(paths: &Paths, profile: Option<&str>) -> anyhow::Result<ExitCode> {
    let host = Host::start(paths, profile, OrphanPolicy::Ask).await?;
    for warning in &host.warnings {
        anstream::eprintln!("warning: {warning}");
    }
    // Errors must not skip stop(), or the units would outlive geco.
    let result = print_until_shutdown(&host, &Client::new(paths.socket())).await;
    // After a hangup stderr fails with EIO; eprintln! would panic and skip stop().
    let _ = writeln!(std::io::stderr(), "stopping units…");
    host.stop().await;
    result.map(|()| ExitCode::SUCCESS)
}

async fn print_until_shutdown(host: &Host, client: &Client) -> anyhow::Result<()> {
    let mut logs = client
        .open(Call::Logs {
            units: Vec::new(),
            tail: 0,
            follow: true,
        })
        .await?;
    let mut states = client.open(Call::SubscribeState).await?;
    let mut out = anstream::stdout();
    let wait = host.wait();
    tokio::pin!(wait);
    loop {
        tokio::select! {
            () = &mut wait => return Ok(()),
            reply = logs.next() => match reply? {
                Some(Reply::Log(record)) => output::write_log(&mut out, &record)?,
                Some(_) => {}
                None => return Ok(()),
            },
            reply = states.next() => match reply? {
                Some(Reply::State(status)) => output::write_state(&mut out, &status)?,
                Some(_) => {}
                None => return Ok(()),
            },
        }
    }
}

async fn up_detached(
    paths: &Paths,
    client: &Client,
    profile: Option<&str>,
) -> anyhow::Result<ExitCode> {
    if client.call(Call::Status).await.is_ok() {
        bail!("a geco supervisor is already running; see `geco status`");
    }
    let orphans = orphaned_groups(&paths.process_state());
    if !orphans.is_empty() {
        if !confirm(&orphan_prompt(&orphans))? {
            return Ok(ExitCode::FAILURE);
        }
        stop_orphans(&paths.process_state(), &orphans).await;
    }
    std::fs::create_dir_all(paths.state_dir())?;
    let log = std::fs::File::create(paths.supervisor_log())?;
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command.arg("__supervise");
    if let Some(profile) = profile {
        command.args(["--profile", profile]);
    }
    let child = command
        .current_dir(&paths.root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log)
        // A separate process group keeps Ctrl-C in this terminal away from the supervisor.
        .process_group(0)
        .spawn()?;
    for _ in 0..100 {
        if client.call(Call::Status).await.is_ok() {
            anstream::println!(
                "geco supervisor running (pid {}); `geco status` shows progress, `geco down` stops it",
                child.id()
            );
            return Ok(ExitCode::SUCCESS);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    bail!(
        "the supervisor did not come up; see {}",
        paths.supervisor_log().display()
    )
}

async fn supervise(paths: &Paths, profile: Option<&str>) -> anyhow::Result<ExitCode> {
    let host = Host::start(paths, profile, OrphanPolicy::Abort).await?;
    for warning in &host.warnings {
        eprintln!("warning: {warning}");
    }
    host.wait().await;
    host.stop().await;
    Ok(ExitCode::SUCCESS)
}

async fn down(paths: &Paths, client: &Client) -> anyhow::Result<ExitCode> {
    match client.call(Call::Shutdown).await {
        Ok(_) => {}
        Err(IpcError::NotRunning) => {
            anstream::println!("no geco supervisor is running");
            return Ok(ExitCode::SUCCESS);
        }
        Err(err) => return Err(err.into()),
    }
    let _ = writeln!(std::io::stderr(), "stopping units…");
    // The socket stops answering right away; the lock is released only once every unit stopped.
    for _ in 0..1200 {
        if InstanceLock::try_acquire(&paths.lock())?.is_some() {
            return Ok(ExitCode::SUCCESS);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    bail!("the supervisor is still stopping after two minutes")
}

async fn status(paths: &Paths, client: &Client, json: bool) -> anyhow::Result<ExitCode> {
    let units = match client.call(Call::Status).await {
        Ok(Reply::Status(units)) => units,
        Ok(other) => bail!("unexpected reply from the supervisor: {other:?}"),
        Err(IpcError::NotRunning) => return fallback::status(paths, json).await,
        Err(err) => return Err(err.into()),
    };
    let mut out = anstream::stdout();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&units)?)?;
    } else {
        output::write_status_table(&mut out, &units, Utc::now())?;
    }
    Ok(if units.iter().all(|unit| unit.state.is_satisfied()) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

async fn logs(
    paths: &Paths,
    client: &Client,
    units: &[String],
    tail: usize,
    follow: bool,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let call = Call::Logs {
        units: units.to_vec(),
        tail,
        follow,
    };
    let mut stream = match client.open(call).await {
        Ok(stream) => stream,
        Err(IpcError::NotRunning) => return fallback::logs(paths, units, tail, follow, json).await,
        Err(err) => return Err(err.into()),
    };
    let mut out = anstream::stdout();
    while let Some(reply) = stream.next().await? {
        match reply {
            Reply::Log(record) if json => writeln!(out, "{}", serde_json::to_string(&record)?)?,
            Reply::Log(record) => output::write_log(&mut out, &record)?,
            Reply::Error(message) => bail!(message),
            _ => {}
        }
    }
    Ok(ExitCode::SUCCESS)
}

async fn simple(client: &Client, call: Call) -> anyhow::Result<ExitCode> {
    match client.call(call).await {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(IpcError::NotRunning) => {
            bail!("no geco supervisor is running; start one with `geco up`")
        }
        Err(err) => Err(err.into()),
    }
}
