use std::{io, sync::Arc, time::Duration};

use chrono::Utc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream, unix::OwnedWriteHalf},
    sync::broadcast::error::RecvError,
};
use tokio_util::sync::CancellationToken;

use super::{Call, PROTOCOL_VERSION, Reply, Request};
use crate::{
    logs::Filter,
    supervisor::{Supervisor, SupervisorError},
};

pub async fn serve(
    listener: UnixListener,
    supervisor: Arc<Supervisor>,
    shutdown: CancellationToken,
) {
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
        let supervisor = Arc::clone(&supervisor);
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            let _ = handle(stream, supervisor, shutdown).await;
        });
    }
}

async fn handle(
    stream: UnixStream,
    supervisor: Arc<Supervisor>,
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
    }
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
    for status in supervisor.status() {
        send(write, &Reply::State(status)).await?;
    }
    loop {
        let received = tokio::select! {
            () = shutdown.cancelled() => return Ok(()),
            received = live.recv() => received,
        };
        match received {
            Ok(status) => send(write, &Reply::State(status)).await?,
            Err(RecvError::Lagged(_)) => {
                for status in supervisor.status() {
                    send(write, &Reply::State(status)).await?;
                }
            }
            Err(RecvError::Closed) => return Ok(()),
        }
    }
}
