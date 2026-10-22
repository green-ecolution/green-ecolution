use std::{collections::BTreeMap, sync::Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::sync::lock;

/// A cargo build seen in a unit's output; geco makes cargo print its progress bar into the
/// pipe so the build count is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Build {
    pub unit: String,
    pub started: DateTime<Utc>,
    pub state: BuildState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum BuildState {
    Running {
        done: u32,
        total: u32,
        active: Vec<String>,
    },
    Finished {
        at: DateTime<Utc>,
    },
    Failed {
        at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// `Compiling foo v1.0`; a build may begin before cargo draws its first bar.
    Step,
    Progress {
        done: u32,
        total: u32,
        active: Vec<String>,
    },
    Finished,
    Failed,
}

/// Classifies one output segment that is already free of ANSI sequences.
pub fn classify(segment: &str) -> Option<Signal> {
    let text = segment.trim();
    if let Some(rest) = text.strip_prefix("Building ") {
        return parse_progress(rest);
    }
    match text.split_once(' ').map_or(text, |(verb, _)| verb) {
        "Compiling" | "Checking" => Some(Signal::Step),
        "Finished" => Some(Signal::Finished),
        _ if text.starts_with("error: could not compile") => Some(Signal::Failed),
        _ => None,
    }
}

/// `[=====>   ] 491/562: serde, tokio, js-…`; cargo cuts the crate list to its bar width.
fn parse_progress(rest: &str) -> Option<Signal> {
    let (_, rest) = rest.trim_start().strip_prefix('[')?.split_once("] ")?;
    let (count, crates) = rest.split_once(':').unwrap_or((rest, ""));
    let (done, total) = count.trim().split_once('/')?;
    let active = crates
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty() && !name.ends_with('…'))
        .map(str::to_owned)
        .collect();
    Some(Signal::Progress {
        done: done.parse().ok()?,
        total: total.parse().ok()?,
        active,
    })
}

fn is_running(build: &Build) -> bool {
    matches!(build.state, BuildState::Running { .. })
}

#[derive(Default)]
struct Tracked {
    pending: Option<DateTime<Utc>>,
    build: Option<Build>,
}

pub struct Builds {
    units: Mutex<BTreeMap<String, Tracked>>,
    tx: watch::Sender<Vec<Build>>,
}

impl Default for Builds {
    fn default() -> Self {
        Self {
            units: Mutex::default(),
            tx: watch::Sender::new(Vec::new()),
        }
    }
}

impl Builds {
    pub fn observe(&self, unit: &str, signal: Signal, now: DateTime<Utc>) {
        let mut units = lock(&self.units);
        let tracked = units.entry(unit.to_owned()).or_default();
        let running = tracked.build.as_ref().is_some_and(is_running);
        match signal {
            Signal::Step => {
                if !running {
                    tracked.pending.get_or_insert(now);
                }
                return;
            }
            Signal::Progress {
                done,
                total,
                active,
            } => {
                let state = BuildState::Running {
                    done,
                    total,
                    active,
                };
                match &mut tracked.build {
                    Some(build) if running => build.state = state,
                    slot => {
                        *slot = Some(Build {
                            unit: unit.to_owned(),
                            started: tracked.pending.take().unwrap_or(now),
                            state,
                        });
                    }
                }
            }
            Signal::Finished | Signal::Failed => {
                tracked.pending = None;
                let Some(build) = tracked.build.as_mut().filter(|_| running) else {
                    return;
                };
                build.state = if signal == Signal::Finished {
                    BuildState::Finished { at: now }
                } else {
                    BuildState::Failed { at: now }
                };
            }
        }
        self.publish(&units);
    }

    /// Drops a build the unit can no longer finish; a finished one stays until it is replaced.
    pub fn abandon(&self, unit: &str) {
        let mut units = lock(&self.units);
        let Some(tracked) = units.get_mut(unit) else {
            return;
        };
        tracked.pending = None;
        if tracked.build.as_ref().is_some_and(is_running) {
            tracked.build = None;
            self.publish(&units);
        }
    }

    fn publish(&self, units: &BTreeMap<String, Tracked>) {
        let builds = units
            .values()
            .filter_map(|tracked| tracked.build.clone())
            .collect();
        self.tx.send_replace(builds);
    }

    pub fn subscribe(&self) -> watch::Receiver<Vec<Build>> {
        self.tx.subscribe()
    }

    pub fn current(&self) -> Vec<Build> {
        self.tx.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::logs::strip_ansi;

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-22T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            + TimeDelta::seconds(seconds)
    }

    fn progress(done: u32, total: u32, active: &[&str]) -> Signal {
        Signal::Progress {
            done,
            total,
            active: active.iter().map(|name| (*name).to_owned()).collect(),
        }
    }

    #[test]
    fn reads_the_progress_bar_bacon_passes_through() {
        let segment = "\u{1b}[1m\u{1b}[96m    Building\u{1b}[0m [====================>    ] 491/562: futures-executor, ppv-lite86, tokio-rustls, js-…";
        assert_eq!(
            classify(&strip_ansi(segment)),
            Some(progress(
                491,
                562,
                &["futures-executor", "ppv-lite86", "tokio-rustls"]
            ))
        );
    }

    #[test]
    fn reads_a_padded_progress_bar_with_one_crate() {
        let segment = "    Building [=======================> ] 556/562: server            ";
        assert_eq!(classify(segment), Some(progress(556, 562, &["server"])));
    }

    #[test]
    fn reads_a_progress_bar_without_crates() {
        assert_eq!(
            classify("    Building [====>   ] 3/10"),
            Some(progress(3, 10, &[]))
        );
    }

    #[test]
    fn recognises_steps_and_the_outcome() {
        assert_eq!(
            classify("   Compiling server v0.8.0 (/repo)"),
            Some(Signal::Step)
        );
        assert_eq!(classify("    Checking memchr v2.8.0"), Some(Signal::Step));
        assert_eq!(
            classify("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.78s"),
            Some(Signal::Finished)
        );
        assert_eq!(
            classify("error: could not compile `server` (lib) due to 1 previous error"),
            Some(Signal::Failed)
        );
    }

    #[test]
    fn ignores_everything_else() {
        for line in [
            "",
            "     Running `target/debug/green-ecolution`",
            r#"{"level":"INFO","message":"Finished request"}"#,
            "error[E0425]: cannot find value `x` in this scope",
            "    Building something unrelated",
            "    Building [===] x/10",
        ] {
            assert_eq!(classify(line), None, "{line}");
        }
    }

    #[test]
    fn a_build_appears_with_its_first_bar_and_starts_at_its_first_step() {
        let builds = Builds::default();
        builds.observe("backend", Signal::Step, at(0));
        assert!(builds.current().is_empty());

        builds.observe("backend", progress(1, 10, &["serde"]), at(2));
        builds.observe("backend", progress(4, 10, &["tokio"]), at(3));

        assert_eq!(
            builds.current(),
            vec![Build {
                unit: "backend".into(),
                started: at(0),
                state: BuildState::Running {
                    done: 4,
                    total: 10,
                    active: vec!["tokio".into()],
                },
            }]
        );
    }

    #[test]
    fn a_build_without_a_bar_never_appears() {
        let builds = Builds::default();
        builds.observe("backend", Signal::Step, at(0));
        builds.observe("backend", Signal::Finished, at(1));
        assert!(builds.current().is_empty());
    }

    #[test]
    fn finished_and_failed_builds_keep_their_start() {
        let builds = Builds::default();
        builds.observe("backend", progress(1, 10, &[]), at(0));
        builds.observe("backend", Signal::Finished, at(8));
        builds.observe("migrations", progress(1, 10, &[]), at(1));
        builds.observe("migrations", Signal::Failed, at(5));

        assert_eq!(
            builds.current(),
            vec![
                Build {
                    unit: "backend".into(),
                    started: at(0),
                    state: BuildState::Finished { at: at(8) },
                },
                Build {
                    unit: "migrations".into(),
                    started: at(1),
                    state: BuildState::Failed { at: at(5) },
                },
            ]
        );
    }

    #[test]
    fn the_next_build_replaces_the_finished_one() {
        let builds = Builds::default();
        builds.observe("backend", progress(1, 10, &[]), at(0));
        builds.observe("backend", Signal::Finished, at(8));
        builds.observe("backend", Signal::Step, at(20));
        builds.observe("backend", progress(1, 3, &["server"]), at(21));

        let current = builds.current();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].started, at(20));
        assert!(matches!(
            current[0].state,
            BuildState::Running { total: 3, .. }
        ));
    }

    #[test]
    fn abandon_drops_a_running_build_but_keeps_a_finished_one() {
        let builds = Builds::default();
        builds.observe("backend", progress(1, 10, &[]), at(0));
        builds.observe("migrations", progress(1, 10, &[]), at(0));
        builds.observe("migrations", Signal::Failed, at(2));

        builds.abandon("backend");
        builds.abandon("migrations");

        let units: Vec<_> = builds.current().into_iter().map(|b| b.unit).collect();
        assert_eq!(units, ["migrations"]);
    }

    #[test]
    fn abandon_forgets_a_pending_start() {
        let builds = Builds::default();
        builds.observe("backend", Signal::Step, at(0));
        builds.abandon("backend");
        builds.observe("backend", progress(1, 10, &[]), at(30));
        assert_eq!(builds.current()[0].started, at(30));
    }

    #[test]
    fn subscribers_see_every_change() {
        let builds = Builds::default();
        let mut live = builds.subscribe();
        builds.observe("backend", progress(1, 10, &[]), at(0));
        assert!(live.has_changed().unwrap());
        assert_eq!(live.borrow_and_update().len(), 1);
        builds.observe("backend", Signal::Step, at(1));
        assert!(!live.has_changed().unwrap());
    }
}
