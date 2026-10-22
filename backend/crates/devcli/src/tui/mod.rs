mod app;
mod input;
mod palette;
mod requests;
mod view;
pub(crate) mod widgets;

use std::{io::Write, process::ExitCode, time::Duration};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyEventKind, MouseButton,
    MouseEvent, MouseEventKind,
};
use futures::StreamExt;
use ratatui::{DefaultTerminal, layout::Rect};
use tokio::{sync::mpsc, time::MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use self::app::{App, Effect, LOG_CAPACITY, Mode, Msg, Pane};
use crate::{
    host::{Host, OrphanPolicy, termination},
    ipc::{Call, Client, IpcError, Reply},
    logs::trace::Trace,
    paths::Paths,
    state::ProcessRegistry,
    tasks::Recipe,
};

/// Attaches to a running supervisor, or hosts one for the lifetime of the TUI.
pub async fn run(paths: &Paths, profile: Option<&str>) -> anyhow::Result<ExitCode> {
    let client = Client::new(paths.socket());
    let host = match client.call(Call::Status).await {
        Ok(_) => None,
        Err(IpcError::NotRunning) => Some(Host::start(paths, profile, OrphanPolicy::Ask).await?),
        Err(err) => return Err(err.into()),
    };
    let mut warnings = host
        .as_ref()
        .map(|h| h.warnings.clone())
        .unwrap_or_default();
    if host.is_none()
        && let Some(requested) = profile
        && let Ok(Reply::Profiles(list)) = client.call(Call::Profiles).await
        && list.active != requested
    {
        warnings.push(format!(
            "the running supervisor uses profile `{}`; switch with p or `geco profile {requested}`",
            list.active
        ));
    }
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
    // panic also stops the process groups this TUI is hosting and releases the mouse, which
    // ratatui's restore leaves captured.
    let mut terminal = ratatui::try_init()?;
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(registry) = &registry {
            registry.terminate_all_now();
        }
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        previous(info);
    }));
    let result = match crossterm::execute!(std::io::stdout(), EnableMouseCapture) {
        Ok(()) => event_loop(&mut terminal, client, shutdown, warnings).await,
        Err(err) => Err(err.into()),
    };
    let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
    // restore() reports its own failure with eprintln!, which panics once the terminal is gone.
    let _ = ratatui::try_restore();
    result
}

fn mouse_msg(
    terminal: &DefaultTerminal,
    mode: Mode,
    mouse: MouseEvent,
) -> anyhow::Result<Option<Msg>> {
    let msg: fn(Pane) -> Msg = match mouse.kind {
        MouseEventKind::ScrollDown => |pane| Msg::Scroll { pane, down: true },
        MouseEventKind::ScrollUp => |pane| Msg::Scroll { pane, down: false },
        MouseEventKind::Down(MouseButton::Left) => Msg::Focus,
        _ => return Ok(None),
    };
    let area = Rect::from(terminal.size()?);
    Ok(view::pane_at(area, mode, mouse.column, mouse.row).map(msg))
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
    let mut findings = client.open(Call::SubscribeFindings).await?;
    let mut logs = client
        .open(Call::Logs {
            units: Vec::new(),
            tail: LOG_CAPACITY,
            follow: true,
            filter: String::new(),
        })
        .await?;
    if let Ok(Reply::Profiles(list)) = client.call(Call::Profiles).await {
        app.update(Msg::Profiles(list.available));
    }
    let mut keys = EventStream::new();
    let mut ticks = tokio::time::interval(Duration::from_secs(1));
    let (flash_tx, mut flash_rx) = mpsc::unbounded_channel::<String>();
    let (trace_tx, mut trace_rx) = mpsc::unbounded_channel::<Trace>();
    let (profiles_tx, mut profiles_rx) = mpsc::unbounded_channel::<Vec<String>>();
    let (recipes_tx, mut recipes_rx) = mpsc::unbounded_channel::<Vec<Recipe>>();
    fetch_recipes(client, &recipes_tx, &flash_tx);

    let mut frames = tokio::time::interval(Duration::from_millis(33));
    frames.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut termination = std::pin::pin!(termination());
    let mut dirty = true;

    loop {
        let connected = !app.disconnected;
        let msg = tokio::select! {
            reply = states.next(), if connected => match reply? {
                Some(Reply::State(status)) => Msg::State(status),
                Some(Reply::Status(units)) => Msg::Units(units),
                Some(Reply::Profile(name)) => {
                    // geco.toml may have gained or lost profiles since the last switch.
                    let client = client.clone();
                    let profiles = profiles_tx.clone();
                    tokio::spawn(async move {
                        if let Ok(Reply::Profiles(list)) = client.call(Call::Profiles).await {
                            let _ = profiles.send(list.available);
                        }
                    });
                    Msg::Profile(name)
                }
                Some(_) => continue,
                None => Msg::Disconnected,
            },
            reply = findings.next(), if connected => match reply? {
                Some(Reply::Findings(list)) => Msg::Findings(list),
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
                Some(Ok(Event::Mouse(mouse))) => match mouse_msg(terminal, app.mode, mouse)? {
                    Some(msg) => msg,
                    None => continue,
                },
                Some(Ok(_)) => continue,
                Some(Err(err)) => return Err(err.into()),
                None => return Ok(()),
            },
            Some(text) = flash_rx.recv() => Msg::Flash(text),
            Some(trace) = trace_rx.recv() => Msg::Trace(trace),
            Some(profiles) = profiles_rx.recv() => Msg::Profiles(profiles),
            Some(recipes) = recipes_rx.recv() => Msg::Recipes(recipes),
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
        let user_input = matches!(msg, Msg::Key(_) | Msg::Scroll { .. } | Msg::Focus(_));
        let effect = app.update(msg);
        if user_input {
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
                    match client.call(call).await {
                        Ok(Reply::Switched(change)) if !change.warnings.is_empty() => {
                            let _ = flash.send(change.warnings.join("; "));
                        }
                        Ok(_) => {}
                        Err(err) => {
                            let _ = flash.send(err.to_string());
                        }
                    }
                });
            }
            Some(Effect::FetchTrace(request_id)) => {
                let client = client.clone();
                let traces = trace_tx.clone();
                let flash = flash_tx.clone();
                tokio::spawn(async move {
                    match client.request_logs(&request_id).await {
                        Ok(records) => {
                            let _ = traces.send(Trace::build(&request_id, &records));
                        }
                        Err(err) => {
                            let _ = flash.send(err.to_string());
                        }
                    }
                });
            }
            Some(Effect::FetchRecipes) => fetch_recipes(client, &recipes_tx, &flash_tx),
            Some(Effect::Run {
                recipe,
                args,
                confirmed,
            }) => {
                let client = client.clone();
                let flash = flash_tx.clone();
                tokio::spawn(async move {
                    // The log reaches the TUI through its own log stream.
                    if let Err(err) = client.run(&recipe, &args, confirmed, |_| Ok(())).await {
                        let _ = flash.send(err.to_string());
                    }
                });
            }
            None => {}
        }
    }
}

fn fetch_recipes(
    client: &Client,
    recipes: &mpsc::UnboundedSender<Vec<Recipe>>,
    flash: &mpsc::UnboundedSender<String>,
) {
    let client = client.clone();
    let recipes = recipes.clone();
    let flash = flash.clone();
    tokio::spawn(async move {
        match client.call(Call::Recipes).await {
            Ok(Reply::Recipes(list)) => {
                let _ = recipes.send(list);
            }
            Ok(_) => {}
            Err(err) => {
                let _ = flash.send(err.to_string());
            }
        }
    });
}
