pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};

pub use client::{Client, IpcError, ReplyStream, RunEnd};

use std::sync::Arc;

use crate::{
    doctor::{Finding, Report, service::Doctor},
    logs::{LogRecord, builds::Build},
    profiles::{ProfileList, Profiles},
    supervisor::{ProfileChange, Supervisor, UnitStatus},
    tasks::{Recipe, Tasks},
};

pub const PROTOCOL_VERSION: u32 = 6;

#[derive(Clone)]
pub struct Services {
    pub supervisor: Arc<Supervisor>,
    pub doctor: Arc<Doctor>,
    pub profiles: Arc<Profiles>,
    pub tasks: Arc<Tasks>,
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
    /// Streams the whole build board whenever it changes.
    SubscribeBuilds,
    Profiles,
    Recipes,
    Run {
        recipe: String,
        args: Vec<String>,
        confirmed: bool,
    },
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
    Recipes(Vec<Recipe>),
    Builds(Vec<Build>),
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
        logs::{
            LogStore,
            builds::{BuildState, Signal},
            parse_line,
        },
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
        fixture_with(|tasks| tasks).await
    }

    async fn fixture_with(
        customize: impl FnOnce(crate::tasks::Tasks) -> crate::tasks::Tasks,
    ) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("geco.sock");
        let units: Vec<Arc<dyn Unit>> =
            vec![Arc::new(FakeUnit::service("db", &Journal::default()))];
        let graph = Graph::new([("db".to_owned(), Vec::<String>::new())]).unwrap();
        let supervisor = Supervisor::new(units, graph, LogStore::new(100));
        let shutdown = CancellationToken::new();
        let listener = UnixListener::bind(&socket).unwrap();
        let tasks = Arc::new(customize(
            crate::tasks::Tasks::new(
                dir.path().to_path_buf(),
                crate::state::ProcessRegistry::in_memory(),
                Arc::clone(&supervisor),
            )
            .with_recipes(Arc::new(crate::tasks::StaticRecipes(
                crate::tasks::catalog::tests::DUMP.to_owned(),
            )))
            .with_program("echo"),
        ));
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
            Arc::clone(&tasks),
        );
        std::fs::write(
            dir.path().join("geco.toml"),
            "[profile.minimal]\nunits = []\n[profile.full]\nunits = []\n[tasks]\ndestructive = [\"migrate-reset\"]\n",
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
                tasks,
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
    async fn recipes_are_listed_with_their_destructive_flag() {
        let f = fixture().await;
        let Reply::Recipes(recipes) = f.client.call(Call::Recipes).await.unwrap() else {
            panic!("expected recipes");
        };
        let reset = recipes.iter().find(|r| r.name == "migrate-reset").unwrap();
        assert!(reset.destructive);
        assert!(recipes.iter().any(|r| r.name == "_compose" && r.private));
    }

    #[tokio::test]
    async fn run_streams_the_task_log_and_finishes() {
        let f = fixture().await;
        let mut lines = Vec::new();
        f.client
            .run("migrate-new", &["x".into()], false, |record| {
                lines.push(record.message);
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(lines, ["$ echo migrate-new x", "migrate-new x"]);
    }

    #[tokio::test]
    async fn run_refuses_destructive_and_unknown_recipes() {
        let f = fixture().await;
        let err = f
            .client
            .run("migrate-reset", &[], false, |_| Ok(()))
            .await
            .unwrap_err();
        assert!(
            matches!(&err, IpcError::Remote(m) if m == "`echo migrate-reset` is destructive and needs confirmation"),
            "{err:?}"
        );
        let err = f
            .client
            .run("nope", &[], false, |_| Ok(()))
            .await
            .unwrap_err();
        assert!(
            matches!(&err, IpcError::Remote(m) if m == "unknown recipe `nope`"),
            "{err:?}"
        );
        f.client
            .run("migrate-reset", &[], true, |_| Ok(()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn tasks_appear_on_the_state_stream() {
        let f = fixture().await;
        let mut stream = f.client.open(Call::SubscribeState).await.unwrap();
        let Some(Reply::Profile(_)) = stream.next().await.unwrap() else {
            panic!("expected the profile first");
        };
        let Some(Reply::State(_)) = stream.next().await.unwrap() else {
            panic!("expected the db state");
        };

        f.client
            .run("migrate-up", &[], false, |_| Ok(()))
            .await
            .unwrap();

        let Some(Reply::State(starting)) = stream.next().await.unwrap() else {
            panic!("expected the task");
        };
        assert_eq!(
            (starting.name.as_str(), starting.origin),
            ("migrate-up", crate::units::Origin::Task)
        );
        let Some(Reply::State(done)) = stream.next().await.unwrap() else {
            panic!("expected the task to finish");
        };
        assert_eq!(done.state, UnitState::Done);
    }

    #[tokio::test]
    async fn build_subscriptions_start_with_the_board_and_follow_it() {
        let f = fixture().await;
        let builds = f.supervisor.logs().builds();
        builds.observe("db", progress(1, 4), Utc::now());

        let mut stream = f.client.open(Call::SubscribeBuilds).await.unwrap();
        let Some(Reply::Builds(initial)) = stream.next().await.unwrap() else {
            panic!("expected the current builds");
        };
        assert!(matches!(
            initial[0].state,
            BuildState::Running {
                done: 1,
                total: 4,
                ..
            }
        ));

        builds.observe("db", progress(3, 4), Utc::now());
        let Some(Reply::Builds(changed)) = stream.next().await.unwrap() else {
            panic!("expected the changed builds");
        };
        assert!(matches!(
            changed[0].state,
            BuildState::Running { done: 3, .. }
        ));
    }

    fn progress(done: u32, total: u32) -> Signal {
        Signal::Progress {
            done,
            total,
            active: Vec::new(),
        }
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
    async fn raw_request(f: &Fixture, request: &str) -> Reply {
        let stream = UnixStream::connect(f.dir.path().join("geco.sock"))
            .await
            .unwrap();
        let (read, mut write) = stream.into_split();
        write
            .write_all(format!("{request}\n").as_bytes())
            .await
            .unwrap();
        let line = BufReader::new(read)
            .lines()
            .next_line()
            .await
            .unwrap()
            .unwrap();
        serde_json::from_str(&line).unwrap()
    }

    #[tokio::test]
    async fn other_protocol_versions_are_rejected() {
        let f = fixture().await;
        let reply = raw_request(&f, r#"{"v":99,"call":{"method":"status"}}"#).await;
        assert!(
            matches!(reply, Reply::Error(message) if message.contains("protocol version 99") && message.contains("geco down"))
        );
    }

    #[tokio::test]
    async fn unknown_methods_are_judged_by_the_protocol_version_first() {
        let f = fixture().await;
        let reply = raw_request(&f, r#"{"v":99,"call":{"method":"frobnicate"}}"#).await;
        assert!(
            matches!(&reply, Reply::Error(message) if message.contains("protocol version 99")),
            "{reply:?}"
        );
        let reply = raw_request(
            &f,
            &format!(r#"{{"v":{PROTOCOL_VERSION},"call":{{"method":"frobnicate"}}}}"#),
        )
        .await;
        assert!(
            matches!(&reply, Reply::Error(message) if message.starts_with("invalid request: ")),
            "{reply:?}"
        );
    }

    /// Blocks in `dump` until the test opens it, like a slow `just --dump`.
    struct GatedRecipes {
        entered: tokio::sync::mpsc::UnboundedSender<()>,
        gate: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
    }

    impl crate::tasks::RecipeSource for GatedRecipes {
        fn dump(&self) -> Result<String, crate::tasks::CatalogError> {
            let _ = self.entered.send(());
            self.gate
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .map_err(|_| crate::tasks::CatalogError::Just("the gate stayed shut".into()))?;
            Ok(crate::tasks::catalog::tests::DUMP.to_owned())
        }
    }

    #[tokio::test]
    async fn a_slow_recipe_listing_does_not_stall_other_calls() {
        let (entered_tx, mut entered) = tokio::sync::mpsc::unbounded_channel();
        let (open, gate) = std::sync::mpsc::channel();
        let f = fixture_with(|tasks| {
            tasks.with_recipes(Arc::new(GatedRecipes {
                entered: entered_tx,
                gate: std::sync::Mutex::new(gate),
            }))
        })
        .await;
        let recipes = tokio::spawn({
            let client = f.client.clone();
            async move { client.call(Call::Recipes).await }
        });
        entered.recv().await.unwrap();

        let status =
            tokio::time::timeout(Duration::from_secs(2), f.client.call(Call::Status)).await;
        open.send(()).unwrap();

        assert!(matches!(status, Ok(Ok(Reply::Status(_)))), "{status:?}");
        let recipes = recipes.await.unwrap();
        assert!(matches!(recipes, Ok(Reply::Recipes(_))), "{recipes:?}");
    }

    #[tokio::test]
    async fn an_interrupt_before_the_task_is_listed_still_stops_it() {
        let (entered_tx, mut entered) = tokio::sync::mpsc::unbounded_channel();
        let (open, gate) = std::sync::mpsc::channel();
        let f = fixture_with(|tasks| {
            tasks
                .with_program("sleep 30; echo")
                .with_recipes(Arc::new(GatedRecipes {
                    entered: entered_tx,
                    gate: std::sync::Mutex::new(gate),
                }))
        })
        .await;
        let (interrupt, mut interrupts) = tokio::sync::mpsc::unbounded_channel();
        let run = tokio::spawn({
            let client = f.client.clone();
            async move {
                client
                    .run_interruptible(
                        "migrate-up",
                        &[],
                        false,
                        "migrate-up",
                        &mut interrupts,
                        |_| Ok(()),
                    )
                    .await
            }
        });
        entered.recv().await.unwrap();
        interrupt.send(()).unwrap();
        // Lets the first stop find nothing before the task gets listed.
        tokio::time::sleep(Duration::from_millis(100)).await;
        open.send(()).unwrap();

        let end = tokio::time::timeout(Duration::from_secs(10), run).await;
        assert!(matches!(end, Ok(Ok(Ok(RunEnd::Stopped)))), "{end:?}");
        assert!(
            f.supervisor.status().iter().all(|s| s.name != "migrate-up"),
            "{:?}",
            f.supervisor.status()
        );
        f.supervisor.stop_all().await;
    }

    #[tokio::test]
    async fn a_second_interrupt_stops_waiting_for_the_stop() {
        let f = fixture_with(|tasks| tasks.with_program("sleep 30; echo")).await;
        let (interrupt, mut interrupts) = tokio::sync::mpsc::unbounded_channel();
        interrupt.send(()).unwrap();
        interrupt.send(()).unwrap();

        let end = tokio::time::timeout(
            Duration::from_secs(5),
            f.client.run_interruptible(
                "migrate-up",
                &[],
                false,
                "migrate-up",
                &mut interrupts,
                |_| Ok(()),
            ),
        )
        .await;

        assert!(matches!(end, Ok(Ok(RunEnd::Abandoned))), "{end:?}");
        f.supervisor.stop_all().await;
    }

    #[tokio::test]
    async fn a_lagging_run_stream_says_how_many_lines_it_skipped() {
        let f = fixture_with(|tasks| tasks.with_program("sleep 0.3; echo")).await;
        let mut stream = f
            .client
            .open(Call::Run {
                recipe: "migrate-up".into(),
                args: Vec::new(),
                confirmed: false,
            })
            .await
            .unwrap();
        let Some(Reply::Log(first)) = stream.next().await.unwrap() else {
            panic!("expected the command line");
        };
        assert_eq!(first.message, "$ sleep 0.3; echo migrate-up");
        for line in 0..5000 {
            f.supervisor.logs().push(parse_line(
                "migrate-up",
                LogFormat::Plain,
                &format!("line {line}"),
                Utc::now(),
            ));
        }

        let mut messages = Vec::new();
        loop {
            match stream.next().await.unwrap() {
                Some(Reply::Log(record)) => messages.push(record.message),
                Some(Reply::Done) => break,
                other => panic!("unexpected reply {other:?}"),
            }
        }
        let skipped = messages
            .iter()
            .find(|message| message.contains("lines skipped"))
            .expect("the gap is reported");
        assert!(skipped.ends_with("see `geco logs migrate-up`"), "{skipped}");
    }

    #[tokio::test]
    async fn shutdown_is_honoured_across_protocol_versions() {
        let f = fixture().await;
        let reply = raw_request(&f, r#"{"v":99,"call":{"method":"shutdown"}}"#).await;
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
