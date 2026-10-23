use std::{future::Future, io, path::PathBuf, pin::Pin};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc,
};

use super::{Call, PROTOCOL_VERSION, Reply, Request};
use crate::{logs::LogRecord, tasks::STOPPED};

#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("no geco supervisor is running for this repository")]
    NotRunning,
    #[error("{0}")]
    Remote(String),
    #[error("the supervisor closed the connection")]
    Closed,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("invalid message on the geco socket: {0}")]
    Protocol(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Client {
    socket: PathBuf,
}

impl Client {
    /// Every buffered record whose request id starts with `request_id`, oldest first.
    pub async fn request_logs(&self, request_id: &str) -> Result<Vec<LogRecord>, IpcError> {
        if request_id.is_empty()
            || request_id.contains(|c: char| c.is_whitespace() || c == ',' || c == '"')
        {
            return Err(IpcError::Remote(format!(
                "request id `{request_id}` cannot be looked up: it is empty or contains whitespace, a comma or a quote"
            )));
        }
        let mut stream = self
            .open(Call::Logs {
                units: Vec::new(),
                tail: usize::MAX,
                follow: false,
                filter: format!("req:{request_id}"),
            })
            .await?;
        let mut records = Vec::new();
        while let Some(reply) = stream.next().await? {
            match reply {
                Reply::Log(record) => records.push(record),
                Reply::Error(message) => return Err(IpcError::Remote(message)),
                _ => {}
            }
        }
        Ok(records)
    }

    pub fn new(socket: PathBuf) -> Self {
        Self { socket }
    }

    pub async fn open(&self, call: Call) -> Result<ReplyStream, IpcError> {
        let stream = UnixStream::connect(&self.socket)
            .await
            .map_err(|err| match err.kind() {
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => IpcError::NotRunning,
                _ => IpcError::Io(err),
            })?;
        let (read, mut write) = stream.into_split();
        let mut line = serde_json::to_vec(&Request {
            v: PROTOCOL_VERSION,
            call,
        })?;
        line.push(b'\n');
        write.write_all(&line).await?;
        Ok(ReplyStream {
            lines: BufReader::new(read).lines(),
            _write: write,
        })
    }

    /// For calls that are answered with exactly one reply.
    pub async fn call(&self, call: Call) -> Result<Reply, IpcError> {
        match self.open(call).await?.next().await? {
            Some(Reply::Error(message)) => Err(IpcError::Remote(message)),
            Some(reply) => Ok(reply),
            None => Err(IpcError::Closed),
        }
    }

    /// Hands each log record of the task to `on_log` and resolves once the task ended.
    pub async fn run(
        &self,
        recipe: &str,
        args: &[String],
        confirmed: bool,
        on_log: impl FnMut(LogRecord) -> io::Result<()>,
    ) -> Result<(), IpcError> {
        let (_interrupt, mut interrupts) = mpsc::unbounded_channel();
        self.run_interruptible(recipe, args, confirmed, recipe, &mut interrupts, on_log)
            .await
            .map(drop)
    }

    /// Like `run`, but each message on `interrupts` asks the supervisor to stop `task`, the
    /// name the recipe runs under; the second one gives up waiting for the stop.
    pub async fn run_interruptible(
        &self,
        recipe: &str,
        args: &[String],
        confirmed: bool,
        task: &str,
        interrupts: &mut mpsc::UnboundedReceiver<()>,
        mut on_log: impl FnMut(LogRecord) -> io::Result<()>,
    ) -> Result<RunEnd, IpcError> {
        let mut stream = self
            .open(Call::Run {
                recipe: recipe.to_owned(),
                args: args.to_vec(),
                confirmed,
            })
            .await?;
        let mut stop = StopRequests::default();
        let mut pending: Option<Pin<Box<dyn Future<Output = ()> + Send + '_>>> = None;
        loop {
            tokio::select! {
                reply = stream.next() => match reply? {
                    Some(Reply::Log(record)) => {
                        on_log(record)?;
                        stop.saw_output();
                    }
                    Some(Reply::Done) => return stop.ended(Ok(())),
                    Some(Reply::Error(message)) => return stop.ended(Err(message)),
                    Some(_) => {}
                    None => return Err(IpcError::Closed),
                },
                Some(()) = interrupts.recv() => {
                    if !stop.interrupt() {
                        return Ok(RunEnd::Abandoned);
                    }
                }
                () = async {
                    if let Some(call) = pending.as_mut() {
                        call.await;
                    }
                }, if pending.is_some() => {
                    pending = None;
                    stop.answered();
                }
            }
            if stop.due() {
                let call = self.call(Call::Stop {
                    unit: task.to_owned(),
                });
                pending = Some(Box::pin(async move {
                    let _ = call.await;
                }));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEnd {
    Done,
    /// Interrupted, and the supervisor reported the task as stopped.
    Stopped,
    /// Interrupted, but the task ended on its own before a stop reached it.
    Ended(Result<(), String>),
    /// Interrupted twice; the stop may still be under way.
    Abandoned,
}

/// A stop sent before the supervisor listed the task finds nothing, or an older failed run of
/// the same name, so it is sent once more after the task's first output: a task logs its
/// command line first thing.
#[derive(Debug, Default)]
struct StopRequests {
    interrupted: bool,
    output_seen: bool,
    in_flight: bool,
    /// Whether the last stop went out after the task's first output.
    sent_after_output: Option<bool>,
}

impl StopRequests {
    /// `false` on the second interrupt.
    fn interrupt(&mut self) -> bool {
        !std::mem::replace(&mut self.interrupted, true)
    }

    fn saw_output(&mut self) {
        self.output_seen = true;
    }

    fn answered(&mut self) {
        self.in_flight = false;
    }

    /// Whether a stop has to go out now; it then counts as in flight.
    fn due(&mut self) -> bool {
        let due = self.interrupted
            && !self.in_flight
            && self
                .sent_after_output
                .is_none_or(|after_output| !after_output && self.output_seen);
        if due {
            self.in_flight = true;
            self.sent_after_output = Some(self.output_seen);
        }
        due
    }

    fn ended(&self, result: Result<(), String>) -> Result<RunEnd, IpcError> {
        match result {
            result if !self.interrupted => result.map(|()| RunEnd::Done).map_err(IpcError::Remote),
            Err(message) if message.ends_with(STOPPED) => Ok(RunEnd::Stopped),
            result => Ok(RunEnd::Ended(result)),
        }
    }
}

pub struct ReplyStream {
    lines: Lines<BufReader<OwnedReadHalf>>,
    _write: OwnedWriteHalf,
}

impl ReplyStream {
    /// Cancel-safe, because `Lines::next_line` is; usable inside `tokio::select!`.
    pub async fn next(&mut self) -> Result<Option<Reply>, IpcError> {
        match self.lines.next_line().await? {
            Some(line) => Ok(Some(serde_json::from_str(&line)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stopped() -> String {
        format!("`just lint` {}", crate::tasks::STOPPED)
    }

    #[test]
    fn nothing_is_stopped_before_an_interrupt() {
        let mut stop = StopRequests::default();
        stop.saw_output();
        assert!(!stop.due());
    }

    #[test]
    fn a_stop_that_went_out_before_any_output_is_sent_again_once_the_task_logs() {
        let mut stop = StopRequests::default();
        assert!(stop.interrupt());
        assert!(stop.due());
        assert!(!stop.due(), "one stop at a time");
        stop.answered();
        assert!(!stop.due(), "nothing new happened");
        stop.saw_output();
        assert!(stop.due());
        stop.answered();
        stop.saw_output();
        assert!(!stop.due(), "that stop reached the listed task");
    }

    #[test]
    fn output_while_the_first_stop_is_under_way_sends_another_once_it_is_answered() {
        let mut stop = StopRequests::default();
        stop.interrupt();
        assert!(stop.due());
        stop.saw_output();
        assert!(!stop.due());
        stop.answered();
        assert!(stop.due());
    }

    #[test]
    fn a_stop_after_output_is_sent_once() {
        let mut stop = StopRequests::default();
        stop.saw_output();
        stop.interrupt();
        assert!(stop.due());
        stop.answered();
        stop.saw_output();
        assert!(!stop.due());
    }

    #[test]
    fn the_second_interrupt_gives_up() {
        let mut stop = StopRequests::default();
        assert!(stop.interrupt());
        assert!(!stop.interrupt());
    }

    #[test]
    fn only_the_stopped_error_of_an_interrupted_run_counts_as_stopped() {
        let quiet = StopRequests::default();
        assert!(matches!(quiet.ended(Ok(())), Ok(RunEnd::Done)));
        assert!(matches!(quiet.ended(Err(stopped())), Err(IpcError::Remote(m)) if m == stopped()));

        let mut stop = StopRequests::default();
        stop.interrupt();
        assert!(matches!(stop.ended(Err(stopped())), Ok(RunEnd::Stopped)));
        assert!(matches!(stop.ended(Ok(())), Ok(RunEnd::Ended(Ok(())))));
        assert!(matches!(
            stop.ended(Err("`just lint` failed: exit status: 1".into())),
            Ok(RunEnd::Ended(Err(m))) if m.ends_with("exit status: 1")
        ));
    }
}
