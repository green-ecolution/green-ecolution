use std::{io, path::PathBuf};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use super::{Call, PROTOCOL_VERSION, Reply, Request};
use crate::logs::LogRecord;

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
