use chrono::{DateTime, Local, TimeDelta, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Clear, LineGauge, List, ListItem, ListState, Paragraph, Wrap},
};

use super::{
    app::{App, LogScope, Mode, Pane},
    help::{Help, HelpLine},
    palette::Palette,
    requests::RequestOrder,
    widgets,
};
use crate::{
    doctor::Severity,
    logs::{
        Level, LogRecord,
        builds::{Build, BuildState},
        trace::{self, TraceLine, TraceLineKind},
    },
    theme,
};

const UNIT_COLORS: [Color; theme::UNIT_SLOTS] = [
    Color::Cyan,
    Color::Magenta,
    Color::Blue,
    Color::Green,
    Color::Yellow,
    Color::LightCyan,
];

/// How long a finished or failed build stays in the panel.
const BUILD_LINGER: TimeDelta = TimeDelta::seconds(5);

const BUILD_NAME_WIDTH: usize = 14;

/// Returns the list pane, the detail pane, the build panel and the status line.
fn areas(area: Rect, mode: Mode, builds: &[&Build]) -> [Rect; 4] {
    let rows: usize = builds.iter().map(|build| build_rows(build)).sum();
    let panel = if rows == 0 {
        0
    } else {
        u16::try_from(rows + 2).unwrap_or(u16::MAX)
    };
    let [main, panel, status] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(panel),
        Constraint::Length(1),
    ])
    .areas(area);
    let list = match mode {
        Mode::Logs => Constraint::Length(32),
        Mode::Requests => Constraint::Percentage(40),
        Mode::Doctor => Constraint::Percentage(45),
    };
    let [list, detail] = Layout::horizontal([list, Constraint::Min(20)]).areas(main);
    [list, detail, panel, status]
}

fn visible_builds(builds: &[Build], now: DateTime<Utc>) -> Vec<&Build> {
    builds
        .iter()
        .filter(|build| match build.state {
            BuildState::Running { .. } => true,
            BuildState::Finished { at } | BuildState::Failed { at } => now - at < BUILD_LINGER,
        })
        .collect()
}

fn build_rows(build: &Build) -> usize {
    match &build.state {
        BuildState::Running { active, .. } if !active.is_empty() => 2,
        _ => 1,
    }
}

pub fn pane_at(area: Rect, mode: Mode, builds: &[Build], column: u16, row: u16) -> Option<Pane> {
    let [list, detail, _, _] = areas(area, mode, &visible_builds(builds, Utc::now()));
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
    let now = Utc::now();
    let builds = visible_builds(&app.builds, now);
    let [list, detail, panel, status] = areas(frame.area(), app.mode, &builds);
    if !builds.is_empty() {
        draw_builds(frame, panel, &builds, now);
    }
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
    if let Some(palette) = &app.palette {
        draw_palette(frame, frame.area(), palette);
    }
    if let Some(help) = &app.help {
        draw_help(frame, frame.area(), help);
    }
}

fn draw_builds(frame: &mut Frame, area: Rect, builds: &[&Build], now: DateTime<Utc>) {
    let block = Block::bordered()
        .title(" Build ")
        .border_style(Style::new().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let dim = Style::new().fg(Color::DarkGray);
    let rows = inner.rows().collect::<Vec<_>>();
    let mut rows = rows.into_iter();
    for build in builds {
        let Some(row) = rows.next() else {
            break;
        };
        let name = Span::styled(
            format!("{:<BUILD_NAME_WIDTH$}", build.unit),
            Style::new().fg(UNIT_COLORS[theme::unit_slot(&build.unit)]),
        );
        match &build.state {
            BuildState::Running {
                done,
                total,
                active,
            } => {
                let counter = format!(" {done}/{total}  {}", duration(now - build.started));
                let width = u16::try_from(counter.chars().count()).unwrap_or(u16::MAX);
                let [gauge, counter_area] =
                    Layout::horizontal([Constraint::Min(10), Constraint::Length(width)]).areas(row);
                let ratio = if *total == 0 {
                    0.0
                } else {
                    (f64::from(*done) / f64::from(*total)).min(1.0)
                };
                frame.render_widget(
                    LineGauge::default()
                        .label(name)
                        .ratio(ratio)
                        .filled_symbol(symbols::line::THICK_HORIZONTAL)
                        .filled_style(Style::new().fg(Color::Yellow))
                        .unfilled_style(dim),
                    gauge,
                );
                frame.render_widget(Paragraph::new(counter), counter_area);
                if !active.is_empty()
                    && let Some(row) = rows.next()
                {
                    let indent = " ".repeat(BUILD_NAME_WIDTH + 1);
                    frame.render_widget(
                        Paragraph::new(format!("{indent}{}", active.join(", "))).style(dim),
                        row,
                    );
                }
            }
            BuildState::Finished { at } => {
                let outcome = Span::styled(
                    format!("✓ finished in {}", duration(*at - build.started)),
                    Style::new().fg(Color::Green),
                );
                frame.render_widget(Line::from(vec![name, Span::raw(" "), outcome]), row);
            }
            BuildState::Failed { at } => {
                let outcome = Span::styled(
                    format!("✗ failed after {}", duration(*at - build.started)),
                    Style::new().fg(Color::Red),
                );
                frame.render_widget(Line::from(vec![name, Span::raw(" "), outcome]), row);
            }
        }
    }
}

fn duration(delta: TimeDelta) -> String {
    let millis = delta.num_milliseconds().max(0);
    if millis < 60_000 {
        let tenths = millis / 100;
        format!("{}.{}s", tenths / 10, tenths % 10)
    } else {
        let seconds = millis / 1000;
        format!("{}m {:02}s", seconds / 60, seconds % 60)
    }
}

fn draw_palette(frame: &mut Frame, area: Rect, palette: &Palette) {
    let matches = palette.matches();
    let rows = u16::try_from(matches.len()).unwrap_or(u16::MAX).min(12);
    let width = area.width.saturating_sub(4).min(90);
    let height = rows.saturating_add(3).min(area.height);
    let popup = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    };
    let block = Block::bordered()
        .title(" Commands ")
        .border_style(Style::new().fg(Color::Green));
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let [query, list] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    frame.render_widget(
        Paragraph::new(format!("> {}", palette.input().text())),
        query,
    );
    frame.set_cursor_position(Position::new(
        query
            .x
            .saturating_add(2)
            .saturating_add(palette.input().cursor_column()),
        query.y,
    ));
    let dim = Style::new().fg(Color::DarkGray);
    let items: Vec<ListItem> = matches
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::raw(entry.label.clone()),
                Span::styled(format!("  {}", entry.hint), dim),
            ]))
        })
        .collect();
    let list_widget =
        List::new(items).highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default().with_selected(Some(palette.selected()));
    frame.render_stateful_widget(list_widget, list, &mut state);
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
            let marker = match app
                .unit_findings(&unit.name)
                .iter()
                .map(|finding| finding.severity)
                .max()
            {
                Some(severity) => Span::styled(" ⚑", Style::new().fg(severity_style(severity).1)),
                None => Span::raw("  "),
            };
            ListItem::new(widgets::unit_line(unit, marker, now))
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
    if let Some(prompt) = &app.prompt {
        let head = format!("run {} · {}: ", prompt.recipe().name, prompt.param().name);
        let mut spans = vec![
            Span::raw(head.clone()),
            Span::raw(prompt.input().text().to_owned()),
        ];
        if let Some(error) = prompt.error() {
            spans.push(Span::styled(
                format!("  {error}"),
                Style::new().fg(Color::Red),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
        let offset = u16::try_from(head.chars().count()).unwrap_or(u16::MAX);
        frame.set_cursor_position(Position::new(
            area.x
                .saturating_add(offset)
                .saturating_add(prompt.input().cursor_column()),
            area.y,
        ));
        return;
    }
    if let Some(run) = &app.pending_run {
        frame.render_widget(
            Paragraph::new(format!("run `{}`? (destructive) [y/N]", run.command))
                .style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
            area,
        );
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
    let ready = app.units.iter().filter(|u| u.state.is_satisfied()).count();
    let mode = if app.hosted { "hosting" } else { "attached" };
    let mut head = vec![mode.to_owned()];
    if let Some(profile) = &app.profile {
        head.push(profile.clone());
    }
    head.push(format!("{ready}/{} ready", app.units.len()));
    let mut parts = vec![head.join(" · ")];
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
        && let Some(finding) = app.unit_finding(&unit.name)
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
    frame.render_widget(
        Paragraph::new(parts.join(" · ")).style(Style::new().add_modifier(Modifier::REVERSED)),
        area,
    );
}

fn draw_help(frame: &mut Frame, area: Rect, help: &Help) {
    let lines = help.lines();
    let key_width = lines
        .iter()
        .filter_map(|line| match line {
            HelpLine::Key(key, _) => Some(key.chars().count()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        + 2;
    let text_width = lines
        .iter()
        .map(|line| match line {
            HelpLine::Heading(title) => title.chars().count(),
            HelpLine::Key(_, what) => key_width + what.chars().count(),
            HelpLine::Blank => 0,
        })
        .max()
        .unwrap_or(0);
    // Borders plus one column of padding on each side.
    let width = u16::try_from(text_width + 4)
        .unwrap_or(u16::MAX)
        .min(area.width);
    let height = u16::try_from(lines.len() + 2)
        .unwrap_or(u16::MAX)
        .min(area.height);
    let popup = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    };
    let visible = usize::from(height.saturating_sub(2));
    let scroll = help.scroll().min(lines.len().saturating_sub(visible));
    let title = if visible < lines.len() {
        format!(
            " Keys {}–{}/{} ",
            scroll + 1,
            (scroll + visible).min(lines.len()),
            lines.len()
        )
    } else {
        " Keys ".to_owned()
    };
    let block = Block::bordered()
        .title(title)
        .title_bottom(" ?/Esc close ")
        .border_style(Style::new().fg(Color::Green));
    let inner = block.inner(popup).inner(Margin::new(1, 0));
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let key_style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
    let text: Vec<Line> = lines
        .iter()
        .skip(scroll)
        .map(|line| match line {
            HelpLine::Heading(title) => {
                Line::styled(*title, Style::new().add_modifier(Modifier::BOLD))
            }
            HelpLine::Key(key, what) => Line::from(vec![
                Span::styled(format!("{key:<key_width$}"), key_style),
                Span::raw(*what),
            ]),
            HelpLine::Blank => Line::default(),
        })
        .collect();
    frame.render_widget(Paragraph::new(text), inner);
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
        tasks::{Param, ParamKind, Recipe},
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

    #[test]
    fn the_status_line_names_the_profile_and_the_palette_lists_actions() {
        let mut app = App::new(true);
        app.update(Msg::Profiles(vec!["full".into(), "minimal".into()]));
        app.update(Msg::Profile("minimal".into()));
        assert!(render(&app).contains("hosting · minimal · 0/0 ready"));

        app.update(Msg::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::NONE,
        )));
        let screen = render(&app);
        assert!(screen.contains("profile full"), "{screen}");
        assert!(screen.contains("profile minimal"), "{screen}");
        assert!(screen.contains("> profile"), "{screen}");
    }

    #[test]
    fn question_mark_shows_the_keys_of_the_mode_in_a_scrollable_box() {
        let mut app = App::new(true);
        let press = |app: &mut App, code| {
            app.update(Msg::Key(KeyEvent::new(code, KeyModifiers::NONE)));
        };
        press(&mut app, KeyCode::Char('?'));
        let screen = render(&app);
        assert!(screen.contains("Keys 1–8/21"), "{screen}");
        assert!(screen.contains("show the selected unit's logs"), "{screen}");
        assert!(screen.contains("? keys"), "{screen}");

        press(&mut app, KeyCode::Char('G'));
        let screen = render(&app);
        assert!(screen.contains("Keys 14–21/21"), "{screen}");
        assert!(screen.contains("quit"), "{screen}");

        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('d'));
        press(&mut app, KeyCode::Char('?'));
        let screen = render(&app);
        assert!(screen.contains("Doctor"), "{screen}");
        assert!(screen.contains("deep check"), "{screen}");
    }

    #[test]
    fn the_status_line_asks_for_parameters_and_destructive_runs() {
        let mut app = App::new(true);
        let new = Recipe {
            name: "migrate-new".into(),
            doc: None,
            group: None,
            private: false,
            destructive: false,
            params: vec![Param {
                name: "name".into(),
                default: None,
                optional: false,
                kind: ParamKind::Singular,
            }],
        };
        let reset = Recipe {
            name: "migrate-reset".into(),
            destructive: true,
            params: Vec::new(),
            ..new.clone()
        };
        app.update(Msg::Recipes(vec![new, reset]));
        for code in [KeyCode::Char(':')]
            .into_iter()
            .chain("migrate-new".chars().map(KeyCode::Char))
            .chain([KeyCode::Enter])
        {
            app.update(Msg::Key(KeyEvent::new(code, KeyModifiers::NONE)));
        }
        assert!(
            render(&app).contains("run migrate-new · name: "),
            "{}",
            render(&app)
        );

        app.update(Msg::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        for code in [KeyCode::Char(':')]
            .into_iter()
            .chain("migrate-reset".chars().map(KeyCode::Char))
            .chain([KeyCode::Enter])
        {
            app.update(Msg::Key(KeyEvent::new(code, KeyModifiers::NONE)));
        }
        assert!(
            render(&app).contains("run `just migrate-reset`? (destructive) [y/N]"),
            "{}",
            render(&app)
        );
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

    fn build(unit: &str, started: DateTime<Utc>, state: BuildState) -> Build {
        Build {
            unit: unit.into(),
            started,
            state,
        }
    }

    #[test]
    fn the_build_panel_appears_only_while_something_builds() {
        let mut app = App::new(false);
        assert!(!render(&app).contains("Build"));

        app.update(Msg::Builds(vec![build(
            "backend",
            Utc::now() - TimeDelta::seconds(12),
            BuildState::Running {
                done: 491,
                total: 562,
                active: vec!["server".into(), "domain-wasm".into()],
            },
        )]));
        let screen = render(&app);

        assert!(screen.contains("┌ Build "), "{screen}");
        assert!(screen.contains("backend"), "{screen}");
        assert!(screen.contains("491/562  12."), "{screen}");
        assert!(screen.contains("server, domain-wasm"), "{screen}");
    }

    #[test]
    fn ended_builds_linger_briefly_with_their_outcome() {
        let mut app = App::new(false);
        let now = Utc::now();
        let ago = |seconds| now - TimeDelta::seconds(seconds);
        app.update(Msg::Builds(vec![
            build("backend", ago(10), BuildState::Finished { at: ago(2) }),
            build("migrations", ago(6), BuildState::Failed { at: ago(2) }),
            build(
                "build-release",
                ago(60),
                BuildState::Finished { at: ago(30) },
            ),
        ]));
        let screen = render(&app);

        assert!(screen.contains("✓ finished in 8.0s"), "{screen}");
        assert!(screen.contains("✗ failed after 4.0s"), "{screen}");
        assert!(!screen.contains("build-release"), "{screen}");
    }

    #[test]
    fn the_build_panel_is_no_pane() {
        let area = Rect::new(0, 0, 110, 10);
        let running = build(
            "backend",
            Utc::now(),
            BuildState::Running {
                done: 1,
                total: 2,
                active: Vec::new(),
            },
        );
        let builds = [running];
        assert_eq!(pane_at(area, Mode::Logs, &builds, 5, 5), Some(Pane::Units));
        assert_eq!(pane_at(area, Mode::Logs, &builds, 5, 7), None);
    }

    #[test]
    fn durations_switch_to_minutes_after_a_minute() {
        assert_eq!(duration(TimeDelta::milliseconds(8_249)), "8.2s");
        assert_eq!(duration(TimeDelta::seconds(65)), "1m 05s");
        assert_eq!(duration(TimeDelta::seconds(-1)), "0.0s");
    }

    #[test]
    fn pane_at_maps_mouse_positions_to_panes() {
        let area = Rect::new(0, 0, 110, 10);
        assert_eq!(pane_at(area, Mode::Logs, &[], 5, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, Mode::Logs, &[], 31, 2), Some(Pane::Units));
        assert_eq!(pane_at(area, Mode::Logs, &[], 32, 2), Some(Pane::Logs));
        assert_eq!(pane_at(area, Mode::Logs, &[], 5, 9), None);
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
        assert_eq!(
            pane_at(area, Mode::Requests, &[], 5, 2),
            Some(Pane::Requests)
        );
        assert_eq!(pane_at(area, Mode::Requests, &[], 60, 2), Some(Pane::Trace));
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

    #[test]
    fn the_marker_stays_visible_on_a_long_lived_failed_unit() {
        let mut app = App::new(false);
        let mut unit = status("backend", UnitState::Failed { reason: "x".into() });
        unit.since = chrono::Utc::now() - chrono::Duration::seconds(15);
        app.update(Msg::State(unit));
        app.update(Msg::Findings(vec![
            Finding::new("ports", Severity::Error, "port taken").unit("backend"),
        ]));
        let screen = render(&app);
        let row = screen.lines().find(|l| l.contains("backend")).unwrap();
        assert!(row.contains("⚑"), "{screen}");
    }
}
