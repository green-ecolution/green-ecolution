use chrono::{Local, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use super::app::{App, LogScope};
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

pub fn draw(frame: &mut Frame, app: &App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(frame.area());
    let [units, logs] =
        Layout::horizontal([Constraint::Length(32), Constraint::Min(20)]).areas(main);
    draw_units(frame, units, app);
    draw_logs(frame, logs, app);
    draw_status(frame, status, app);
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

fn draw_units(frame: &mut Frame, area: Rect, app: &App) {
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
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title(" Units "))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
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
        Paragraph::new(lines).block(Block::bordered().title(title)),
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

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.show_help {
        "q quit · j/k select · Tab all/selected · r restart · R restart with dependents · \
         s start/stop · PgUp/PgDn scroll · G follow"
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
                theme::ago(unit.since, Utc::now())
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
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::{
        supervisor::UnitState,
        tui::app::{
            Msg,
            tests::{record, status},
        },
    };

    fn render(app: &App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(110, 10)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
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
}
