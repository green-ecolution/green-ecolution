pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};

pub use client::{Client, IpcError, ReplyStream};

use crate::{logs::LogRecord, supervisor::UnitStatus};

pub const PROTOCOL_VERSION: u32 = 1;

/// One call per connection; clients that need several streams open several connections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Call {
    Status,
    Start {
        unit: String,
    },
    Stop {
        unit: String,
    },
    Restart {
        unit: String,
        cascade: bool,
    },
    Shutdown,
    Logs {
        units: Vec<String>,
        tail: usize,
        follow: bool,
    },
    SubscribeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub v: u32,
    pub call: Call,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Reply {
    Status(Vec<UnitStatus>),
    Done,
    Log(LogRecord),
    State(UnitStatus),
    Error(String),
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use chrono::Utc;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{UnixListener, UnixStream},
    };
    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::{
        config::LogFormat,
        graph::Graph,
        logs::{LogStore, parse_line},
        supervisor::{Supervisor, UnitState},
        units::{
            Unit,
            fake::{FakeUnit, Journal},
        },
    };

    struct Fixture {
        dir: tempfile::TempDir,
        client: Client,
        supervisor: Arc<Supervisor>,
        shutdown: CancellationToken,
    }

    async fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("geco.sock");
        let units: Vec<Arc<dyn Unit>> =
            vec![Arc::new(FakeUnit::service("db", &Journal::default()))];
        let graph = Graph::new([("db".to_owned(), Vec::<String>::new())]).unwrap();
        let supervisor = Supervisor::new(units, graph, LogStore::new(100));
        let shutdown = CancellationToken::new();
        let listener = UnixListener::bind(&socket).unwrap();
        tokio::spawn(server::serve(
            listener,
            Arc::clone(&supervisor),
            shutdown.clone(),
        ));
        Fixture {
            dir,
            client: Client::new(socket),
            supervisor,
            shutdown,
        }
    }

    fn log(message: &str) -> LogRecord {
        parse_line("db", LogFormat::Plain, message, Utc::now())
    }

    #[tokio::test]
    async fn answers_status_requests() {
        let f = fixture().await;
        let Reply::Status(units) = f.client.call(Call::Status).await.unwrap() else {
            panic!("expected a status reply");
        };
        assert_eq!(units[0].name, "db");
        assert_eq!(units[0].state, UnitState::Stopped);
    }

    #[tokio::test]
    async fn supervisor_errors_become_remote_errors() {
        let f = fixture().await;
        let err = f
            .client
            .call(Call::Restart {
                unit: "nope".into(),
                cascade: false,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, IpcError::Remote(message) if message == "unknown unit `nope`"));
    }

    #[tokio::test]
    async fn start_runs_the_unit() {
        let f = fixture().await;
        assert_eq!(
            f.client
                .call(Call::Start { unit: "db".into() })
                .await
                .unwrap(),
            Reply::Done
        );
        tokio::time::timeout(Duration::from_secs(5), async {
            while f.supervisor.status()[0].state != UnitState::Ready {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn streams_the_tail_and_then_follows() {
        let f = fixture().await;
        f.supervisor.logs().push(log("old"));
        let mut stream = f
            .client
            .open(Call::Logs {
                units: Vec::new(),
                tail: 10,
                follow: true,
            })
            .await
            .unwrap();

        let Some(Reply::Log(first)) = stream.next().await.unwrap() else {
            panic!("expected the tail");
        };
        f.supervisor.logs().push(log("new"));
        let Some(Reply::Log(second)) = stream.next().await.unwrap() else {
            panic!("expected a followed record");
        };

        assert_eq!(first.message, "old");
        assert_eq!(second.message, "new");
    }

    #[tokio::test]
    async fn state_subscriptions_start_with_the_current_states() {
        let f = fixture().await;
        let mut stream = f.client.open(Call::SubscribeState).await.unwrap();

        let Some(Reply::State(initial)) = stream.next().await.unwrap() else {
            panic!("expected the current state");
        };
        f.supervisor.start_all();
        let Some(Reply::State(next)) = stream.next().await.unwrap() else {
            panic!("expected a state change");
        };

        assert_eq!(initial.state, UnitState::Stopped);
        assert_eq!(next.state, UnitState::Starting);
    }

    #[tokio::test]
    async fn shutdown_cancels_the_token() {
        let f = fixture().await;
        assert_eq!(f.client.call(Call::Shutdown).await.unwrap(), Reply::Done);
        assert!(f.shutdown.is_cancelled());
    }

    #[tokio::test]
    async fn other_protocol_versions_are_rejected() {
        let f = fixture().await;
        let stream = UnixStream::connect(f.dir.path().join("geco.sock"))
            .await
            .unwrap();
        let (read, mut write) = stream.into_split();
        write
            .write_all(b"{\"v\":99,\"call\":{\"method\":\"status\"}}\n")
            .await
            .unwrap();
        let line = BufReader::new(read)
            .lines()
            .next_line()
            .await
            .unwrap()
            .unwrap();

        let reply: Reply = serde_json::from_str(&line).unwrap();
        assert!(matches!(reply, Reply::Error(message) if message.contains("protocol version 99")));
    }

    #[tokio::test]
    async fn a_missing_socket_means_no_supervisor() {
        let dir = tempfile::tempdir().unwrap();
        let client = Client::new(dir.path().join("geco.sock"));
        assert!(matches!(
            client.call(Call::Status).await,
            Err(IpcError::NotRunning)
        ));
    }
}
