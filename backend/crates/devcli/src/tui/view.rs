use chrono::{DateTime, Local, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};

use super::{
    app::{App, LogScope, Mode, Pane},
    requests::RequestOrder,
};
use crate::{
    doctor::Severity,
    logs::{
        Level, LogRecord,
        trace::{self, TraceLine, TraceLineKind},
    },
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

/// Returns the list pane, the detail pane and the status line.
fn areas(area: Rect, mode: Mode) -> [Rect; 3] {
    let [main, status] = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(area);
    let list = match mode {
        Mode::Logs => Constraint::Length(32),
        Mode::Requests => Constraint::Percentage(40),
        Mode::Doctor => Constraint::Percentage(45),
    };
    let [list, detail] = Layout::horizontal([list, Constraint::Min(20)]).areas(main);
    [list, detail, status]
}

pub fn pane_at(area: Rect, mode: Mode, column: u16, row: u16) -> Option<Pane> {
    let [list, detail, _] = areas(area, mode);
    let (left, right) = match mode {
        Mode::Logs => (Pane::Units, Pane::Logs),
        Mode::Requests => (Pane::Requests, Pane::Trace),
        Mode::Doctor => (Pane::Findings, Pane::Findings),
    };
    let position = Position::new(column, row);
    if list.contains(position) {
        Some(left)
    } else if detail.contains(position) {
        Some(right)
    } else {
        None
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let [list, detail, status] = areas(frame.area(), app.mode);
    let now = Utc::now();
    match app.mode {
        Mode::Logs => {
            draw_units(frame, list, app, now);
            draw_logs(frame, detail, app);
        }
        Mode::Requests => {
            draw_request_list(frame, list, app);
            draw_trace(frame, detail, app);
        }
        Mode::Doctor => {
            draw_findings(frame, list, app);
            draw_finding_detail(frame, detail, app);
        }
    }
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
            let mut spans = vec![
                Span::styled(theme::symbol(tone), color),
                Span::raw(format!(" {:<16} ", unit.name)),
                Span::styled(unit.state.short(), color),
                Span::styled(
                    format!(" {}", theme::ago(unit.since, now)),
                    Style::new().fg(Color::DarkGray),
                ),
            ];
            let worst = app
                .unit_findings(&unit.name)
                .iter()
                .map(|finding| finding.severity)
                .max();
            if let Some(severity) = worst {
                spans.push(Span::styled(
                    " ⚑",
                    Style::new().fg(severity_style(severity).1),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();
    let list = List::new(items)
        .block(pane_block(app, Pane::Units, " Units ".to_owned()))
        .highlight_style(highlight(app, Pane::Units));
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn severity_style(severity: Severity) -> (&'static str, Color) {
    match severity {
        Severity::Error => ("✖", Color::Red),
        Severity::Hint => ("●", Color::Yellow),
    }
}

fn draw_findings(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(" Doctor · {} finding(s) ", app.findings.len());
    if app.findings.is_empty() {
        let block = pane_block(app, Pane::Findings, title);
        frame.render_widget(
            Paragraph::new("no findings; r checks again, R includes the slow checks").block(block),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .findings
        .iter()
        .map(|finding| {
            let (symbol, color) = severity_style(finding.severity);
            ListItem::new(Line::from(vec![
                Span::styled(symbol, Style::new().fg(color)),
                Span::raw(format!(" {}", finding.title)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(pane_block(app, Pane::Findings, title))
        .highlight_style(highlight(app, Pane::Findings));
    let mut state = ListState::default().with_selected(Some(app.finding_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_finding_detail(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered()
        .title(" Detail ")
        .border_style(Style::new().fg(Color::DarkGray));
    let Some(finding) = app.findings.get(app.finding_selected) else {
        frame.render_widget(block, area);
        return;
    };
    let dim = Style::new().fg(Color::DarkGray);
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let origin = match &finding.unit {
        Some(unit) => format!("{} · {unit}", finding.id),
        None => finding.id.clone(),
    };
    let mut lines = vec![
        Line::styled(finding.title.as_str(), bold),
        Line::styled(origin, dim),
        Line::default(),
    ];
    lines.extend(finding.detail.lines().map(Line::raw));
    lines.push(Line::default());
    lines.push(match &finding.fix {
        Some(fix) => Line::from(vec![
            Span::raw("fix: "),
            Span::styled(fix.command.as_str(), bold),
            Span::raw(if fix.destructive {
                " (destructive)"
            } else {
                ""
            }),
            Span::styled("  [f] run", dim),
        ]),
        None => Line::styled("no automatic fix", dim),
    });
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        area,
    );
}

fn highlight(app: &App, pane: Pane) -> Style {
    if app.focus == pane {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::new().bg(Color::DarkGray)
    }
}

fn status_color(status: u16) -> Color {
    match status {
        500.. => Color::Red,
        400..=499 => Color::Yellow,
        _ => Color::Green,
    }
}

fn draw_request_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .requests
        .ordered()
        .into_iter()
        .map(|request| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<6} ", request.method)),
                Span::styled(
                    format!("{} ", request.status),
                    Style::new().fg(status_color(request.status)),
                ),
                Span::styled(
                    format!("{:>6} ", format!("{}ms", request.latency_ms)),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::raw(request.path.clone()),
            ]))
        })
        .collect();
    let title = match app.requests.order {
        RequestOrder::Newest => " Requests · newest first ",
        RequestOrder::Slowest => " Requests · slowest first ",
    };
    let list = List::new(items)
        .block(pane_block(app, Pane::Requests, title.to_owned()))
        .highlight_style(highlight(app, Pane::Requests));
    let mut state = ListState::default().with_selected(app.requests.selected_index());
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_trace(frame: &mut Frame, area: Rect, app: &App) {
    let Some(trace) = &app.requests.trace else {
        let hint = if app.requests.is_empty() {
            "no finished requests yet; the backend logs them at debug level"
        } else {
            "loading…"
        };
        let block = pane_block(app, Pane::Trace, " Trace ".to_owned());
        frame.render_widget(Paragraph::new(hint).block(block), area);
        return;
    };
    let title = match &trace.summary {
        Some(s) => format!(
            " Trace · {} {} · {} · {} ms ",
            s.method, s.path, s.status, s.latency_ms
        ),
        None => format!(" Trace · {} ", trace.request_id),
    };
    let lines: Vec<Line> = trace
        .lines
        .iter()
        .skip(app.requests.trace_scroll)
        .map(trace_line)
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(pane_block(app, Pane::Trace, title)),
        area,
    );
}

fn trace_line(line: &TraceLine) -> Line<'_> {
    let dim = Style::new().fg(Color::DarkGray);
    let mut spans = vec![
        Span::styled(format!("{:>11} ", trace::offset_label(line.offset_us)), dim),
        Span::raw("  ".repeat(line.depth)),
    ];
    match &line.kind {
        TraceLineKind::Enter { span } => spans.push(Span::styled(
            format!("▸ {span}"),
            Style::new().add_modifier(Modifier::BOLD),
        )),
        TraceLineKind::Event { level, message, .. } => {
            spans.push(Span::styled(
                format!("{:<5} ", level.label()),
                level_style(*level),
            ));
            spans.push(Span::raw(message.as_str()));
        }
        TraceLineKind::Close { target, busy, .. } => spans.push(Span::styled(
            format!("◂ {} busy {busy}", target.as_deref().unwrap_or("span")),
            dim,
        )),
    }
    Line::from(spans)
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
    if let Some(input) = &app.input {
        let mut spans = vec![Span::raw(format!("/{}", input.text()))];
        if let Some(error) = &app.filter_error {
            spans.push(Span::styled(
                format!("  {error}"),
                Style::new().fg(Color::Red),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
        frame.set_cursor_position(Position::new(
            area.x
                .saturating_add(1)
                .saturating_add(input.cursor_column()),
            area.y,
        ));
        return;
    }
    if let Some(finding) = &app.pending_fix
        && let Some(fix) = &finding.fix
    {
        let style = if fix.destructive {
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::Yellow)
        };
        let warning = if fix.destructive {
            "(destructive) "
        } else {
            ""
        };
        frame.render_widget(
            Paragraph::new(format!("run `{}`? {warning}[y/N]", fix.command)).style(style),
            area,
        );
        return;
    }
    let text = if app.show_help {
        match app.mode {
            Mode::Logs => {
                "q quit · h/l pane · j/k select or scroll · ^u/^d half page · \
                 g/G oldest/follow · Enter unit logs · Tab all/selected · / filter · \
                 t requests · d doctor · f fix · r restart · R restart with dependents · s start/stop"
            }
            Mode::Doctor => {
                "d/Esc logs · j/k select · f fix · r check again · R deep check · q quit"
            }
            Mode::Requests => {
                "t/Esc logs · h/l pane · j/k select or scroll · Enter open trace · \
                 o newest/slowest · ^u/^d half page · g/G top/bottom · q quit"
            }
        }
        .to_owned()
    } else {
        let ready = app.units.iter().filter(|u| u.state.is_satisfied()).count();
        let mode = if app.hosted { "hosting" } else { "attached" };
        let mut parts = vec![format!("{mode} · {ready}/{} ready", app.units.len())];
        if app.mode == Mode::Requests {
            parts.push(format!("{} requests", app.requests.len()));
        }
        if let Some(unit) = app.selected_unit() {
            parts.push(format!(
                "{}: {} ({})",
                unit.name,
                unit.state,
                theme::ago(unit.since, now)
            ));
        }
        if let Some(unit) = app.selected_unit()
            && let Some(finding) = app.unit_findings(&unit.name).first()
        {
            let action = if finding.fix.is_some() {
                " · [f] fix"
            } else {
                ""
            };
            parts.push(format!("{}{action}", finding.title));
        }
        if !app.findings.is_empty() {
            parts.push(format!("{} finding(s)", app.findings.len()));
        }
        if !app.filter_text.is_empty() {
            parts.push(format!("filter: {}", app.filter_text));
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
        doctor::{Finding, Fix},
        logs::trace::{Trace, tests::finished_request},
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
        assert_eq!(pane_at(area, Mode::Logs, 5, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, Mode::Logs, 31, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, Mode::Logs, 32, 2), Some(Pane::Logs));
        assert_eq!(pane_at(area, Mode::Logs, 5, 9), None);
    }

    #[test]
    fn the_status_line_shows_the_input_and_then_the_active_filter() {
        let mut app = App::new(false);
        app.update(Msg::Key(KeyEvent::new(
            KeyCode::Char('/'),
            KeyModifiers::NONE,
        )));
        for c in "unit:db".chars() {
            app.update(Msg::Key(KeyEvent::new(
                KeyCode::Char(c),
                KeyModifiers::NONE,
            )));
        }
        assert!(render(&app).lines().last().unwrap().starts_with("/unit:db"));

        app.update(Msg::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(render(&app).contains("filter: unit:db"));
    }

    #[test]
    fn an_invalid_filter_is_shown_next_to_the_input_until_escape() {
        let mut app = App::new(false);
        for code in "/level:loud"
            .chars()
            .map(KeyCode::Char)
            .chain([KeyCode::Enter])
        {
            app.update(Msg::Key(KeyEvent::new(code, KeyModifiers::NONE)));
        }
        let screen = render(&app);
        let status = screen.lines().last().unwrap();
        assert!(status.contains("/level:loud"), "{status}");
        assert!(
            status.contains("invalid filter: unknown level `loud`"),
            "{status}"
        );

        app.update(Msg::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(!render(&app).contains("invalid filter"));
    }

    fn press(app: &mut App, code: KeyCode) {
        app.update(Msg::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    }

    #[test]
    fn the_request_mode_lists_requests_and_shows_the_trace() {
        let mut app = App::new(false);
        app.update(Msg::Log(finished_request("r1", "/api/v1/tree", 404, 12)));
        press(&mut app, KeyCode::Char('t'));
        assert!(render(&app).contains("loading…"));

        app.update(Msg::Trace(Trace::build(
            "r1",
            &[finished_request("r1", "/api/v1/tree", 404, 12)],
        )));
        let screen = render(&app);

        assert!(screen.contains("Requests · newest first"), "{screen}");
        assert!(
            screen.contains("GET    404   12ms /api/v1/tree"),
            "{screen}"
        );
        assert!(
            screen.contains("Trace · GET /api/v1/tree · 404 · 12 ms"),
            "{screen}"
        );
        assert!(screen.contains("+0.000ms ▸ http_request"), "{screen}");
        assert!(
            screen.contains("+0.000ms   DEBUG end of stream"),
            "{screen}"
        );
    }

    #[test]
    fn pane_at_follows_the_mode() {
        let area = Rect::new(0, 0, 110, 10);
        assert_eq!(pane_at(area, Mode::Requests, 5, 2), Some(Pane::Requests));
        assert_eq!(pane_at(area, Mode::Requests, 60, 2), Some(Pane::Trace));
    }

    #[test]
    fn the_doctor_view_shows_findings_and_the_selected_detail() {
        let mut app = App::new(false);
        app.update(Msg::State(status("backend", UnitState::Ready)));
        app.update(Msg::Findings(vec![
            Finding::new(
                "dev-db-migrations",
                Severity::Error,
                "dev database is 2 migration(s) behind",
            )
            .unit("backend")
            .detail("Newest missing migration: 20260101.")
            .fix(Fix::just("migrate-up")),
        ]));
        let screen = render(&app);
        assert!(screen.contains("● backend"), "{screen}");
        assert!(screen.contains("⚑"), "{screen}");
        assert!(screen.contains("1 finding(s)"), "{screen}");
        assert!(
            screen.contains("dev database is 2 migration(s) behind · [f] fix"),
            "{screen}"
        );

        app.update(Msg::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        )));
        let screen = render(&app);
        assert!(screen.contains("Doctor · 1 finding(s)"), "{screen}");
        assert!(
            screen.contains("✖ dev database is 2 migration(s) behind"),
            "{screen}"
        );
        assert!(screen.contains("dev-db-migrations · backend"), "{screen}");
        assert!(
            screen.contains("Newest missing migration: 20260101."),
            "{screen}"
        );
        assert!(screen.contains("fix: just migrate-up"), "{screen}");

        app.update(Msg::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::NONE,
        )));
        assert!(
            render(&app)
                .lines()
                .last()
                .unwrap()
                .starts_with("run `just migrate-up`? [y/N]")
        );
    }
}
