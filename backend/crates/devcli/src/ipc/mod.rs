pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};

pub use client::{Client, IpcError, ReplyStream};

use std::sync::Arc;

use crate::{
    doctor::{Finding, Report, service::Doctor},
    logs::LogRecord,
    profiles::{ProfileList, Profiles},
    supervisor::{ProfileChange, Supervisor, UnitStatus},
};

pub const PROTOCOL_VERSION: u32 = 4;

#[derive(Clone)]
pub struct Services {
    pub supervisor: Arc<Supervisor>,
    pub doctor: Arc<Doctor>,
    pub profiles: Arc<Profiles>,
}

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
        /// Filter-language expression; empty matches everything.
        #[serde(default)]
        filter: String,
    },
    SubscribeState,
    /// Runs the cheap checks, plus the deep ones when `deep` is set.
    Doctor {
        deep: bool,
    },
    Fix {
        finding: String,
        confirmed: bool,
    },
    SubscribeFindings,
    Profiles,
    SwitchProfile {
        profile: String,
    },
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
    Report(Report),
    Findings(Vec<Finding>),
    Profiles(ProfileList),
    Switched(ProfileChange),
    Profile(String),
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
        let doctor = crate::doctor::service::Doctor::new(
            vec![
                crate::doctor::fake::FakeCheck::new("ports")
                    .finds(vec![crate::doctor::Finding::new(
                        "ports",
                        crate::doctor::Severity::Error,
                        "port 5432 is taken",
                    )])
                    .arc(),
            ],
            crate::doctor::fake::test_env(dir.path()),
            Arc::clone(&supervisor),
            crate::state::ProcessRegistry::in_memory(),
        );
        std::fs::write(
            dir.path().join("geco.toml"),
            "[profile.minimal]\nunits = []\n[profile.full]\nunits = []\n",
        )
        .unwrap();
        supervisor.set_profile("minimal");
        let profiles = Arc::new(crate::profiles::Profiles::new(
            dir.path().to_path_buf(),
            crate::state::ProcessRegistry::in_memory(),
            Arc::clone(&supervisor),
        ));
        tokio::spawn(server::serve(
            listener,
            Services {
                supervisor: Arc::clone(&supervisor),
                doctor: Arc::clone(&doctor),
                profiles,
            },
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
    async fn doctor_calls_report_and_stream_findings() {
        let f = fixture().await;
        let mut stream = f.client.open(Call::SubscribeFindings).await.unwrap();
        let Some(Reply::Findings(initial)) = stream.next().await.unwrap() else {
            panic!("expected the current findings");
        };
        assert!(initial.is_empty());

        let Reply::Report(report) = f.client.call(Call::Doctor { deep: false }).await.unwrap()
        else {
            panic!("expected a report");
        };
        assert_eq!(report.findings[0].id, "ports");
        let Some(Reply::Findings(changed)) = stream.next().await.unwrap() else {
            panic!("expected changed findings");
        };
        assert_eq!(changed.len(), 1);

        let err = f
            .client
            .call(Call::Fix {
                finding: "ports".into(),
                confirmed: true,
            })
            .await
            .unwrap_err();
        assert!(
            matches!(err, IpcError::Remote(message) if message == "finding `ports` has no automatic fix")
        );
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
                filter: String::new(),
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
    async fn log_streams_apply_the_filter_to_tail_and_follow() {
        let f = fixture().await;
        f.supervisor.logs().push(log("ERROR old"));
        f.supervisor.logs().push(log("old"));
        let mut stream = f
            .client
            .open(Call::Logs {
                units: Vec::new(),
                tail: 10,
                follow: true,
                filter: "level:error".into(),
            })
            .await
            .unwrap();
        f.supervisor.logs().push(log("new"));
        f.supervisor.logs().push(log("ERROR new"));

        let mut seen = Vec::new();
        for _ in 0..2 {
            let Some(Reply::Log(record)) = stream.next().await.unwrap() else {
                panic!("expected a record");
            };
            seen.push(record.message);
        }
        assert_eq!(seen, ["ERROR old", "ERROR new"]);
    }

    #[tokio::test]
    async fn invalid_filters_are_reported() {
        let f = fixture().await;
        let mut stream = f
            .client
            .open(Call::Logs {
                units: Vec::new(),
                tail: 10,
                follow: false,
                filter: "level:loud".into(),
            })
            .await
            .unwrap();
        let Some(Reply::Error(message)) = stream.next().await.unwrap() else {
            panic!("expected an error");
        };
        assert!(
            message.starts_with("invalid filter: unknown level `loud`"),
            "{message}"
        );
    }

    #[tokio::test]
    async fn state_subscriptions_start_with_the_current_states() {
        let f = fixture().await;
        let mut stream = f.client.open(Call::SubscribeState).await.unwrap();
        let Some(Reply::Profile(_)) = stream.next().await.unwrap() else {
            panic!("expected the active profile first");
        };

        let Some(Reply::State(initial)) = stream.next().await.unwrap() else {
            panic!("expected the current state");
        };
        f.supervisor.start_all().await;
        let Some(Reply::State(next)) = stream.next().await.unwrap() else {
            panic!("expected a state change");
        };

        assert_eq!(initial.state, UnitState::Stopped);
        assert_eq!(next.state, UnitState::Starting);
    }

    #[tokio::test]
    async fn profiles_are_listed_and_announced_on_the_state_stream() {
        let f = fixture().await;
        let Reply::Profiles(list) = f.client.call(Call::Profiles).await.unwrap() else {
            panic!("expected profiles");
        };
        assert_eq!(list.active, "minimal");
        assert_eq!(list.available, ["full", "minimal"]);

        let mut stream = f.client.open(Call::SubscribeState).await.unwrap();
        let Some(Reply::Profile(active)) = stream.next().await.unwrap() else {
            panic!("expected the active profile first");
        };
        assert_eq!(active, "minimal");
    }

    #[tokio::test]
    async fn a_switch_replies_with_the_change_and_is_announced_on_the_state_stream() {
        let f = fixture().await;
        let mut stream = f.client.open(Call::SubscribeState).await.unwrap();
        let Some(Reply::Profile(_)) = stream.next().await.unwrap() else {
            panic!("expected the active profile first");
        };
        let Some(Reply::State(_)) = stream.next().await.unwrap() else {
            panic!("expected the current state");
        };

        let Reply::Switched(change) = f
            .client
            .call(Call::SwitchProfile {
                profile: "full".into(),
            })
            .await
            .unwrap()
        else {
            panic!("expected the change");
        };
        assert_eq!(change.profile, "full");

        let Some(Reply::Status(_)) = stream.next().await.unwrap() else {
            panic!("expected the new unit list");
        };
        let Some(Reply::Profile(active)) = stream.next().await.unwrap() else {
            panic!("expected the new profile");
        };
        assert_eq!(active, "full");
    }

    #[tokio::test]
    async fn unknown_profiles_are_refused() {
        let f = fixture().await;
        let err = f
            .client
            .call(Call::SwitchProfile {
                profile: "nope".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, IpcError::Remote(message) if message == "unknown profile `nope`"));
    }

    #[tokio::test]
    async fn shutdown_cancels_the_token() {
        let f = fixture().await;
        assert_eq!(f.client.call(Call::Shutdown).await.unwrap(), Reply::Done);
        assert!(f.shutdown.is_cancelled());
    }

    #[tokio::test]
    async fn shutdown_still_cancels_when_the_client_is_already_gone() {
        use std::io::Write;

        let f = fixture().await;
        // Blocking std socket on the current-thread runtime: the server cannot run before the
        // client has hung up, so its reply write fails.
        let mut stream =
            std::os::unix::net::UnixStream::connect(f.dir.path().join("geco.sock")).unwrap();
        stream
            .write_all(
                format!("{{\"v\":{PROTOCOL_VERSION},\"call\":{{\"method\":\"shutdown\"}}}}\n")
                    .as_bytes(),
            )
            .unwrap();
        drop(stream);

        tokio::time::timeout(Duration::from_secs(5), f.shutdown.cancelled())
            .await
            .unwrap();
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
        assert!(
            matches!(reply, Reply::Error(message) if message.contains("protocol version 99") && message.contains("geco down"))
        );
    }

    #[tokio::test]
    async fn shutdown_is_honoured_across_protocol_versions() {
        let f = fixture().await;
        let stream = UnixStream::connect(f.dir.path().join("geco.sock"))
            .await
            .unwrap();
        let (read, mut write) = stream.into_split();
        write
            .write_all(b"{\"v\":99,\"call\":{\"method\":\"shutdown\"}}\n")
            .await
            .unwrap();
        let line = BufReader::new(read)
            .lines()
            .next_line()
            .await
            .unwrap()
            .unwrap();

        let reply: Reply = serde_json::from_str(&line).unwrap();
        assert!(matches!(reply, Reply::Done));
        tokio::time::timeout(Duration::from_secs(5), f.shutdown.cancelled())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn request_ids_the_filter_would_misread_are_rejected() {
        let f = fixture().await;
        let client = Client::new(f.dir.path().join("geco.sock"));
        for id in ["a b", "a,b", "", "a\"b"] {
            let result = client.request_logs(id).await;
            assert!(
                matches!(&result, Err(IpcError::Remote(m)) if m.contains("cannot be looked up")),
                "{id:?}: {result:?}"
            );
        }
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
