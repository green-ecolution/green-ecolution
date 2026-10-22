mod app;
mod view;

use std::{io::Write, process::ExitCode, time::Duration};

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::{sync::mpsc, time::MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use self::app::{App, Effect, LOG_CAPACITY, Msg};
use crate::{
    host::{Host, OrphanPolicy, termination},
    ipc::{Call, Client, IpcError, Reply},
    paths::Paths,
    state::ProcessRegistry,
};

/// Attaches to a running supervisor, or hosts one for the lifetime of the TUI.
pub async fn run(paths: &Paths, profile: Option<&str>) -> anyhow::Result<ExitCode> {
    let client = Client::new(paths.socket());
    let host = match client.call(Call::Status).await {
        Ok(_) => None,
        Err(IpcError::NotRunning) => Some(Host::start(paths, profile, OrphanPolicy::Ask).await?),
        Err(err) => return Err(err.into()),
    };
    let warnings = host
        .as_ref()
        .map(|h| h.warnings.clone())
        .unwrap_or_default();
    let registry = host.as_ref().map(|h| h.registry.clone());
    let shutdown = host.as_ref().map(|h| h.shutdown.clone());

    let result = run_terminal(&client, shutdown, registry, warnings).await;

    if let Some(host) = host {
        // After a hangup stderr fails with EIO; eprintln! would panic and skip stop().
        let _ = writeln!(std::io::stderr(), "stopping units…");
        host.stop().await;
    }
    result.map(|()| ExitCode::SUCCESS)
}

async fn run_terminal(
    client: &Client,
    shutdown: Option<CancellationToken>,
    registry: Option<ProcessRegistry>,
    warnings: Vec<String>,
) -> anyhow::Result<()> {
    // ratatui::init installs a hook that restores the terminal on panic; ours runs first so a
    // panic also stops the process groups this TUI is hosting.
    let mut terminal = ratatui::try_init()?;
    if let Some(registry) = registry {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            registry.terminate_all_now();
            previous(info);
        }));
    }
    let result = event_loop(&mut terminal, client, shutdown, warnings).await;
    // restore() reports its own failure with eprintln!, which panics once the terminal is gone.
    let _ = ratatui::try_restore();
    result
}

/// `shutdown` is the hosted supervisor's token, `None` when attached to another process.
async fn event_loop(
    terminal: &mut DefaultTerminal,
    client: &Client,
    shutdown: Option<CancellationToken>,
    warnings: Vec<String>,
) -> anyhow::Result<()> {
    let hosted = shutdown.is_some();
    let shutdown = shutdown.unwrap_or_default();
    let mut app = App::new(hosted);
    for warning in warnings {
        app.update(Msg::Flash(warning));
    }
    let mut states = client.open(Call::SubscribeState).await?;
    let mut logs = client
        .open(Call::Logs {
            units: Vec::new(),
            tail: LOG_CAPACITY,
            follow: true,
        })
        .await?;
    let mut keys = EventStream::new();
    let mut ticks = tokio::time::interval(Duration::from_secs(1));
    let (flash_tx, mut flash_rx) = mpsc::unbounded_channel::<String>();

    let mut frames = tokio::time::interval(Duration::from_millis(33));
    frames.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut termination = std::pin::pin!(termination());
    let mut dirty = true;

    loop {
        let connected = !app.disconnected;
        let msg = tokio::select! {
            reply = states.next(), if connected => match reply? {
                Some(Reply::State(status)) => Msg::State(status),
                Some(_) => continue,
                None => Msg::Disconnected,
            },
            reply = logs.next(), if connected => match reply? {
                Some(Reply::Log(record)) => Msg::Log(record),
                Some(_) => continue,
                None => Msg::Disconnected,
            },
            event = keys.next() => match event {
                Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => Msg::Key(key),
                Some(Ok(_)) => continue,
                Some(Err(err)) => return Err(err.into()),
                None => return Ok(()),
            },
            Some(text) = flash_rx.recv() => Msg::Flash(text),
            _ = ticks.tick() => Msg::Tick,
            _ = frames.tick() => {
                if dirty {
                    terminal.draw(|frame| view::draw(frame, &app))?;
                    dirty = false;
                }
                continue;
            }
            () = &mut termination, if hosted => return Ok(()),
            // `geco down` cancels the host; its units must stop without waiting for `q`.
            () = shutdown.cancelled(), if hosted => Msg::HostStopped,
        };
        dirty = true;
        let key_pressed = matches!(msg, Msg::Key(_));
        let effect = app.update(msg);
        if key_pressed {
            terminal.draw(|frame| view::draw(frame, &app))?;
            dirty = false;
        }
        match effect {
            Some(Effect::Quit) => return Ok(()),
            // Restarts can take the full stop grace period; keep the UI responsive meanwhile.
            Some(Effect::Call(call)) => {
                let client = client.clone();
                let flash = flash_tx.clone();
                tokio::spawn(async move {
                    if let Err(err) = client.call(call).await {
                        let _ = flash.send(err.to_string());
                    }
                });
            }
            None => {}
        }
    }
}
