use chrono::{DateTime, Local, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use super::app::{App, LogScope, Pane};
use crate::{
    logs::{Level, LogRecord},
    theme::{self, Tone},
};

const UNIT_COLORS: [Color; theme::UNIT_SLOTS] = [
    Color::Cyan,
    Color::Magenta,
    Color::Blue,
    Color::Green,
    Color::Yellow,
    Color::LightCyan,
];

/// Returns the units pane, the logs pane and the status line.
fn areas(area: Rect) -> [Rect; 3] {
    let [main, status] = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(area);
    let [units, logs] =
        Layout::horizontal([Constraint::Length(32), Constraint::Min(20)]).areas(main);
    [units, logs, status]
}

pub fn pane_at(area: Rect, column: u16, row: u16) -> Option<Pane> {
    let [units, logs, _] = areas(area);
    let position = Position::new(column, row);
    if units.contains(position) {
        Some(Pane::Units)
    } else if logs.contains(position) {
        Some(Pane::Logs)
    } else {
        None
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let [units, logs, status] = areas(frame.area());
    let now = Utc::now();
    draw_units(frame, units, app, now);
    draw_logs(frame, logs, app);
    draw_status(frame, status, app, now);
}

fn pane_block(app: &App, pane: Pane, title: String) -> Block<'static> {
    let block = Block::bordered().title(title);
    if app.focus == pane {
        block
            .border_style(Style::new().fg(Color::Green))
            .title_style(Style::new().add_modifier(Modifier::BOLD))
    } else {
        block.border_style(Style::new().fg(Color::DarkGray))
    }
}

fn tone_color(tone: Tone) -> Color {
    match tone {
        Tone::Good => Color::Green,
        Tone::Busy => Color::Yellow,
        Tone::Bad => Color::Red,
        Tone::Idle => Color::DarkGray,
    }
}

fn level_style(level: Level) -> Style {
    match level {
        Level::Error => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        Level::Warn => Style::new().fg(Color::Yellow),
        Level::Info => Style::new().fg(Color::Green),
        Level::Debug => Style::new().fg(Color::Blue),
        Level::Trace | Level::Unknown => Style::new().fg(Color::DarkGray),
    }
}

fn draw_units(frame: &mut Frame, area: Rect, app: &App, now: DateTime<Utc>) {
    let items: Vec<ListItem> = app
        .units
        .iter()
        .map(|unit| {
            let tone = theme::tone(&unit.state);
            let color = Style::new().fg(tone_color(tone));
            ListItem::new(Line::from(vec![
                Span::styled(theme::symbol(tone), color),
                Span::raw(format!(" {:<18} ", unit.name)),
                Span::styled(unit.state.short(), color),
                Span::styled(
                    format!(" {}", theme::ago(unit.since, now)),
                    Style::new().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();
    let highlight = if app.focus == Pane::Units {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::new().bg(Color::DarkGray)
    };
    let list = List::new(items)
        .block(pane_block(app, Pane::Units, " Units ".to_owned()))
        .highlight_style(highlight);
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_logs(frame: &mut Frame, area: Rect, app: &App) {
    let mut title = match (app.scope, app.selected_unit()) {
        (LogScope::Selected, Some(unit)) => format!(" Logs · {} ", unit.name),
        _ => " Logs · all units ".to_owned(),
    };
    if app.scroll > 0 {
        title.push_str("· scrolled ");
    }
    let height = usize::from(area.height.saturating_sub(2));
    let show_unit = app.scope == LogScope::All;
    let lines: Vec<Line> = app
        .visible_logs(height)
        .into_iter()
        .map(|record| log_line(record, show_unit))
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(pane_block(app, Pane::Logs, title)),
        area,
    );
}

fn log_line(record: &LogRecord, show_unit: bool) -> Line<'_> {
    let time = record.timestamp.with_timezone(&Local).format("%H:%M:%S ");
    let mut spans = vec![Span::styled(
        time.to_string(),
        Style::new().fg(Color::DarkGray),
    )];
    if show_unit {
        let color = UNIT_COLORS[theme::unit_slot(&record.unit)];
        spans.push(Span::styled(
            format!("{:<14} ", record.unit),
            Style::new().fg(color),
        ));
    }
    if record.level != Level::Unknown {
        spans.push(Span::styled(
            format!("{:<5} ", record.level.label()),
            level_style(record.level),
        ));
    }
    spans.push(Span::raw(record.message.as_str()));
    Line::from(spans)
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App, now: DateTime<Utc>) {
    let text = if app.show_help {
        "q quit · h/l pane · j/k select or scroll · ^u/^d half page · g/G oldest/follow · \
         Tab all/selected · r restart · R restart with dependents · s start/stop"
            .to_owned()
    } else {
        let ready = app.units.iter().filter(|u| u.state.is_satisfied()).count();
        let mode = if app.hosted { "hosting" } else { "attached" };
        let mut parts = vec![format!("{mode} · {ready}/{} ready", app.units.len())];
        if let Some(unit) = app.selected_unit() {
            parts.push(format!(
                "{}: {} ({})",
                unit.name,
                unit.state,
                theme::ago(unit.since, now)
            ));
        }
        if let Some(flash) = &app.flash {
            parts.push(flash.clone());
        }
        parts.push("? keys".to_owned());
        parts.join(" · ")
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::new().add_modifier(Modifier::REVERSED)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    use super::*;
    use crate::{
        supervisor::UnitState,
        tui::app::{
            Msg,
            tests::{record, status},
        },
    };

    fn buffer(app: &App) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(110, 10)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn render(app: &App) -> String {
        let buffer = buffer(app);
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn shows_units_logs_and_the_selected_state() {
        let mut app = App::new(true);
        app.update(Msg::State(status("db", UnitState::Ready)));
        app.update(Msg::State(status(
            "backend",
            UnitState::Waiting {
                on: vec!["migrations".into()],
            },
        )));
        app.update(Msg::Log(record("db", "database system is ready")));

        let screen = render(&app);

        assert!(screen.contains("● db"), "{screen}");
        assert!(screen.contains("ready 0s"), "{screen}");
        assert!(screen.contains("◐ backend"), "{screen}");
        assert!(screen.contains("database system is ready"), "{screen}");
        assert!(
            screen.contains("hosting · 1/2 ready · db: ready"),
            "{screen}"
        );
    }

    #[test]
    fn the_status_line_explains_what_a_unit_waits_for() {
        let mut app = App::new(false);
        app.update(Msg::State(status(
            "backend",
            UnitState::Waiting {
                on: vec!["migrations".into()],
            },
        )));

        assert!(render(&app).contains("backend: waiting for migrations"));
    }

    #[test]
    fn the_focused_pane_has_the_highlighted_border() {
        let mut app = App::new(false);
        let units_corner = (0, 0);
        let logs_corner = (32, 0);

        let screen = buffer(&app);
        assert_eq!(screen[units_corner].fg, Color::Green);
        assert_eq!(screen[logs_corner].fg, Color::DarkGray);

        app.update(Msg::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )));
        let screen = buffer(&app);
        assert_eq!(screen[units_corner].fg, Color::DarkGray);
        assert_eq!(screen[logs_corner].fg, Color::Green);
    }

    #[test]
    fn pane_at_maps_mouse_positions_to_panes() {
        let area = Rect::new(0, 0, 110, 10);
        assert_eq!(pane_at(area, 5, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, 31, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, 32, 2), Some(Pane::Logs));
        assert_eq!(pane_at(area, 5, 9), None);
    }
}
