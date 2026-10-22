use std::{io, time::Duration};

use chrono::Utc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream, unix::OwnedWriteHalf},
    sync::broadcast::error::{RecvError, TryRecvError},
};
use tokio_util::sync::CancellationToken;

use super::{Call, PROTOCOL_VERSION, Reply, Request, Services};
use crate::{
    doctor::service::Doctor,
    logs::Filter,
    supervisor::{Supervisor, SupervisorError},
    tasks::Tasks,
};

pub async fn serve(listener: UnixListener, services: Services, shutdown: CancellationToken) {
    loop {
        let accepted = tokio::select! {
            () = shutdown.cancelled() => return,
            accepted = listener.accept() => accepted,
        };
        let Ok((stream, _)) = accepted else {
            // Typically EMFILE; back off instead of spinning.
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        };
        let services = services.clone();
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            let _ = handle(stream, services, shutdown).await;
        });
    }
}

async fn handle(
    stream: UnixStream,
    services: Services,
    shutdown: CancellationToken,
) -> io::Result<()> {
    let (read, mut write) = stream.into_split();
    let Some(line) = BufReader::new(read).lines().next_line().await? else {
        return Ok(());
    };
    let request = match serde_json::from_str::<Request>(&line) {
        Ok(request) if request.v == PROTOCOL_VERSION => request,
        // A newer client must still be able to stop an older supervisor.
        Ok(Request {
            call: Call::Shutdown,
            ..
        }) => {
            let _ = send(&mut write, &Reply::Done).await;
            shutdown.cancel();
            return Ok(());
        }
        Ok(request) => {
            let message = format!(
                "protocol version {} is not supported; this supervisor speaks {PROTOCOL_VERSION}; \
                 restart geco with matching versions (stop the running supervisor with `geco down`)",
                request.v
            );
            return send(&mut write, &Reply::Error(message)).await;
        }
        Err(err) => {
            return send(&mut write, &Reply::Error(format!("invalid request: {err}"))).await;
        }
    };
    let Services {
        supervisor,
        doctor,
        profiles,
        tasks,
    } = services;
    match request.call {
        Call::Status => send(&mut write, &Reply::Status(supervisor.status())).await,
        Call::Start { unit } => {
            let result = supervisor.start(&unit).await;
            send(&mut write, &outcome(result)).await
        }
        Call::Stop { unit } => {
            let result = supervisor.stop(&unit).await;
            send(&mut write, &outcome(result)).await
        }
        Call::Restart { unit, cascade } => {
            let result = supervisor.restart(&unit, cascade).await;
            send(&mut write, &outcome(result)).await
        }
        Call::Shutdown => {
            let _ = send(&mut write, &Reply::Done).await;
            shutdown.cancel();
            Ok(())
        }
        Call::Logs {
            units,
            tail,
            follow,
            filter,
        } => {
            stream_logs(
                &mut write,
                &supervisor,
                &units,
                &filter,
                tail,
                follow,
                &shutdown,
            )
            .await
        }
        Call::SubscribeState => stream_state(&mut write, &supervisor, &shutdown).await,
        Call::Doctor { deep } => send(&mut write, &Reply::Report(doctor.run(deep).await)).await,
        Call::Fix { finding, confirmed } => {
            let reply = match doctor.fix(&finding, confirmed).await {
                Ok(()) => Reply::Done,
                Err(err) => Reply::Error(err.to_string()),
            };
            send(&mut write, &reply).await
        }
        Call::SubscribeFindings => stream_findings(&mut write, &doctor, &shutdown).await,
        Call::Profiles => {
            let reply = match profiles.list() {
                Ok(list) => Reply::Profiles(list),
                Err(err) => Reply::Error(err.to_string()),
            };
            send(&mut write, &reply).await
        }
        Call::SwitchProfile { profile } => {
            let reply = match profiles.switch(&profile).await {
                Ok(change) => Reply::Switched(change),
                Err(err) => Reply::Error(err.to_string()),
            };
            send(&mut write, &reply).await
        }
        Call::Recipes => {
            let reply = match tasks.catalog() {
                Ok(catalog) => Reply::Recipes(catalog.recipes().cloned().collect()),
                Err(err) => Reply::Error(err.to_string()),
            };
            send(&mut write, &reply).await
        }
        Call::Run {
            recipe,
            args,
            confirmed,
        } => run_task(&mut write, &tasks, &supervisor, &recipe, &args, confirmed).await,
    }
}

/// A client that hangs up only ends the stream; the task keeps running in the supervisor.
async fn run_task(
    write: &mut OwnedWriteHalf,
    tasks: &Tasks,
    supervisor: &Supervisor,
    recipe: &str,
    args: &[String],
    confirmed: bool,
) -> io::Result<()> {
    let spec = match tasks.recipe(recipe, args) {
        Ok(spec) => spec,
        Err(err) => return send(write, &Reply::Error(err.to_string())).await,
    };
    let unit = spec.name.clone();
    let mut live = supervisor.logs().subscribe();
    let run = tasks.run(spec, confirmed);
    tokio::pin!(run);
    let result = loop {
        tokio::select! {
            result = &mut run => break result,
            received = live.recv() => match received {
                Ok(record) if record.unit == unit => send(write, &Reply::Log(record)).await?,
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break run.await,
            },
        }
    };
    // A oneshot resolves only after its output was pushed, but those records may still wait here.
    loop {
        match live.try_recv() {
            Ok(record) if record.unit == unit => send(write, &Reply::Log(record)).await?,
            Ok(_) | Err(TryRecvError::Lagged(_)) => {}
            Err(_) => break,
        }
    }
    let reply = match result {
        Ok(()) => Reply::Done,
        Err(err) => Reply::Error(err.to_string()),
    };
    send(write, &reply).await
}

fn outcome(result: Result<(), SupervisorError>) -> Reply {
    match result {
        Ok(()) => Reply::Done,
        Err(err) => Reply::Error(err.to_string()),
    }
}

async fn send(write: &mut OwnedWriteHalf, reply: &Reply) -> io::Result<()> {
    let mut line = serde_json::to_vec(reply).map_err(io::Error::other)?;
    line.push(b'\n');
    write.write_all(&line).await
}

async fn stream_findings(
    write: &mut OwnedWriteHalf,
    doctor: &Doctor,
    shutdown: &CancellationToken,
) -> io::Result<()> {
    let mut live = doctor.subscribe();
    send(write, &Reply::Findings(doctor.findings())).await?;
    loop {
        let received = tokio::select! {
            () = shutdown.cancelled() => return Ok(()),
            received = live.recv() => received,
        };
        match received {
            Ok(findings) => send(write, &Reply::Findings(findings)).await?,
            Err(RecvError::Lagged(_)) => send(write, &Reply::Findings(doctor.findings())).await?,
            Err(RecvError::Closed) => return Ok(()),
        }
    }
}

async fn stream_logs(
    write: &mut OwnedWriteHalf,
    supervisor: &Supervisor,
    units: &[String],
    filter: &str,
    tail: usize,
    follow: bool,
    shutdown: &CancellationToken,
) -> io::Result<()> {
    let filter = match filter.parse::<Filter>() {
        Ok(filter) => filter,
        Err(err) => return send(write, &Reply::Error(format!("invalid filter: {err}"))).await,
    };
    // Subscribing before taking the snapshot and skipping by sequence number closes the gap
    // between the two without sending a record twice.
    let mut live = supervisor.logs().subscribe();
    let snapshot = supervisor.logs().snapshot(units, &filter, tail, Utc::now());
    let mut last_sent = snapshot.last().map_or(0, |record| record.seq);
    for record in snapshot {
        send(write, &Reply::Log(record)).await?;
    }
    if !follow {
        return Ok(());
    }
    loop {
        let received = tokio::select! {
            () = shutdown.cancelled() => return Ok(()),
            received = live.recv() => received,
        };
        match received {
            Ok(record)
                if record.seq > last_sent
                    && (units.is_empty() || units.contains(&record.unit))
                    && filter.matches(&record, Utc::now()) =>
            {
                last_sent = record.seq;
                send(write, &Reply::Log(record)).await?;
            }
            Ok(_) | Err(RecvError::Lagged(_)) => {}
            Err(RecvError::Closed) => return Ok(()),
        }
    }
}

async fn stream_state(
    write: &mut OwnedWriteHalf,
    supervisor: &Supervisor,
    shutdown: &CancellationToken,
) -> io::Result<()> {
    let mut live = supervisor.subscribe();
    let mut profiles = supervisor.subscribe_profile();
    send(write, &Reply::Profile(supervisor.profile())).await?;
    for status in supervisor.status() {
        send(write, &Reply::State(status)).await?;
    }
    loop {
        tokio::select! {
            () = shutdown.cancelled() => return Ok(()),
            received = live.recv() => match received {
                Ok(status) => send(write, &Reply::State(status)).await?,
                Err(RecvError::Lagged(_)) => {
                    for status in supervisor.status() {
                        send(write, &Reply::State(status)).await?;
                    }
                }
                Err(RecvError::Closed) => return Ok(()),
            },
            received = profiles.recv() => match received {
                Ok(profile) => {
                    send(write, &Reply::Status(supervisor.status())).await?;
                    send(write, &Reply::Profile(profile)).await?;
                }
                Err(RecvError::Lagged(_)) => {
                    send(write, &Reply::Status(supervisor.status())).await?;
                    send(write, &Reply::Profile(supervisor.profile())).await?;
                }
                Err(RecvError::Closed) => return Ok(()),
            },
        }
    }
}
