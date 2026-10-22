use std::{collections::BTreeSet, io, time::Duration};

use anyhow::bail;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame, Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, layout::Position,
    text::Span, widgets::Paragraph,
};

use crate::{
    ipc::{Call, Client, Reply},
    supervisor::{UnitState, UnitStatus},
    tui::widgets::unit_line,
    units::Origin,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Starting,
    Ready,
    Failed(Vec<String>),
}

/// A failed unit and everything waiting on it can no longer become ready, so start-up has
/// settled once all other units are ready or done.
pub fn progress(units: &[UnitStatus]) -> Progress {
    let units: Vec<&UnitStatus> = units
        .iter()
        .filter(|unit| unit.origin != Origin::Task)
        .collect();
    let failed: Vec<String> = units
        .iter()
        .filter(|unit| matches!(unit.state, UnitState::Failed { .. }))
        .map(|unit| unit.name.clone())
        .collect();
    let mut blocked: BTreeSet<&str> = failed.iter().map(String::as_str).collect();
    loop {
        let before = blocked.len();
        for unit in &units {
            if let UnitState::Waiting { on } = &unit.state
                && on
                    .iter()
                    .any(|dependency| blocked.contains(dependency.as_str()))
            {
                blocked.insert(unit.name.as_str());
            }
        }
        if blocked.len() == before {
            break;
        }
    }
    let settled = units
        .iter()
        .all(|unit| unit.state.is_satisfied() || blocked.contains(unit.name.as_str()));
    match (settled, failed.is_empty()) {
        (false, _) => Progress::Starting,
        (true, true) => Progress::Ready,
        (true, false) => Progress::Failed(failed),
    }
}

pub fn draw(frame: &mut Frame, units: &[UnitStatus], now: DateTime<Utc>) {
    let lines: Vec<_> = units
        .iter()
        .map(|unit| unit_line(unit, Span::raw("  "), now))
        .collect();
    frame.render_widget(Paragraph::new(lines), frame.area());
}

/// Draws below the prompt without raw mode, so Ctrl-C still arrives as a signal.
pub async fn follow(client: &Client) -> anyhow::Result<Progress> {
    let Reply::Status(mut units) = client.call(Call::Status).await? else {
        bail!("unexpected reply from the supervisor");
    };
    let height = u16::try_from(units.len().max(1))
        .unwrap_or(u16::MAX)
        .min(30);
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(io::stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;
    let mut states = client.open(Call::SubscribeState).await?;
    let mut ticks = tokio::time::interval(Duration::from_secs(1));
    let interrupted = tokio::signal::ctrl_c();
    tokio::pin!(interrupted);
    let outcome = loop {
        terminal.draw(|frame| draw(frame, &units, Utc::now()))?;
        let state = progress(&units);
        if state != Progress::Starting {
            break state;
        }
        tokio::select! {
            reply = states.next() => match reply? {
                Some(Reply::State(status)) => upsert(&mut units, status),
                Some(Reply::Status(all)) => units = all,
                Some(_) => {}
                None => bail!("the supervisor stopped during start-up"),
            },
            _ = ticks.tick() => {}
            _ = &mut interrupted => break Progress::Starting,
        }
    };
    let area = terminal.get_frame().area();
    terminal.set_cursor_position(Position::new(0, area.bottom().saturating_sub(1)))?;
    terminal.show_cursor()?;
    drop(terminal);
    println!();
    Ok(outcome)
}

fn upsert(units: &mut Vec<UnitStatus>, status: UnitStatus) {
    match units.iter_mut().find(|unit| unit.name == status.name) {
        Some(unit) => *unit = status,
        None => units.push(status),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ratatui::{Terminal, TerminalOptions, Viewport, backend::TestBackend};

    use super::*;
    use crate::units::UnitKind;

    fn unit(name: &str, state: UnitState) -> UnitStatus {
        UnitStatus {
            name: name.into(),
            kind: UnitKind::Service,
            origin: Origin::Local,
            state,
            since: Utc::now(),
        }
    }

    fn failed(name: &str) -> UnitStatus {
        unit(name, UnitState::Failed { reason: "x".into() })
    }

    fn waiting(name: &str, on: &[&str]) -> UnitStatus {
        unit(
            name,
            UnitState::Waiting {
                on: on.iter().map(|d| (*d).to_owned()).collect(),
            },
        )
    }

    #[test]
    fn ready_once_every_unit_is_ready_or_done() {
        assert_eq!(
            progress(&[
                unit("db", UnitState::Ready),
                unit("migrations", UnitState::Done)
            ]),
            Progress::Ready
        );
        assert_eq!(
            progress(&[
                unit("db", UnitState::Ready),
                unit("backend", UnitState::Starting)
            ]),
            Progress::Starting
        );
        assert_eq!(progress(&[]), Progress::Ready);
    }

    #[test]
    fn failures_settle_start_up_once_the_rest_is_ready_or_blocked() {
        let units = [
            failed("migrations"),
            waiting("backend", &["migrations"]),
            waiting("frontend", &["backend"]),
            unit("db", UnitState::Ready),
        ];
        assert_eq!(
            progress(&units),
            Progress::Failed(vec!["migrations".into()])
        );

        let still_starting = [failed("migrations"), unit("keycloak", UnitState::Starting)];
        assert_eq!(progress(&still_starting), Progress::Starting);
    }

    #[test]
    fn tasks_do_not_count() {
        let mut task = failed("lint");
        task.origin = Origin::Task;
        assert_eq!(
            progress(&[unit("db", UnitState::Ready), task]),
            Progress::Ready
        );
    }

    #[test]
    fn draws_one_line_per_unit_in_an_inline_viewport() {
        let mut terminal = Terminal::with_options(
            TestBackend::new(40, 4),
            TerminalOptions {
                viewport: Viewport::Inline(2),
            },
        )
        .unwrap();
        let units = [unit("db", UnitState::Ready), waiting("backend", &["db"])];
        terminal
            .draw(|frame| draw(frame, &units, Utc::now()))
            .unwrap();
        let screen: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(screen.contains("db"), "{screen}");
        assert!(screen.contains("backend"), "{screen}");
        assert!(screen.contains("waiting"), "{screen}");
    }
}
